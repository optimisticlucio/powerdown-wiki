use super::structs;
use crate::{
    ServerState, art::structs::Comment, errs::RootErrors, user::{User, UsermadePost}, utils::template_to_response
};
use askama::Template;
use axum::{
    extract::{OriginalUri, Path, Query, State},
    response::{IntoResponse, Response},
};
use comrak::markdown_to_html;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use http::Uri;
use postgres_types::ToSql;

#[derive(Debug, Template)]
#[template(path = "art/page.html")]
struct ArtPage<'a> {
    user: Option<User>,
    original_uri: Uri,
    user_search_params: &'a structs::ArtSearchParameters,

    // Whether or not the user has the permissions to edit the page.
    user_can_edit_page: bool,

    title: String,
    artists: Vec<String>,
    formatted_creation_date: String,
    art_urls: Vec<String>,
    tags: Vec<String>,
    description: Option<String>, // Assumed to be markdown.

    comments: Vec<super::structs::Comment>,

    newer_art_url: Option<String>,
    older_art_url: Option<String>,
}

impl<'a> ArtPage<'a> {
    /// Given a URL, returns true if it's one that should be wrapped in a <video> tag.
    /// Assumes the URL has a file extension. If not, this breaks.
    fn url_is_of_video(&self, url: &&String) -> bool {
        ["mp4", "avi", "mkv", "mov", "wmv", "flv", "m4v"].iter().any(|ext| url.ends_with(ext))
    }
}

pub async fn art_page(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>,
    Query(query_params): Query<structs::ArtSearchParameters>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();
    let user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    if let Some(requested_art) = structs::PageArt::get_by_slug(&db_connection, &art_slug).await {
        let (older_art_url, newer_art_url) =
            get_older_and_newer_art_slugs(&art_slug, &query_params, &db_connection).await;
        let art_urls = requested_art.get_art_urls();

        let user_can_edit_page: bool = user
            .as_ref()
            .is_some_and(|user| requested_art.can_be_modified_by(user));

        let markdownified_description = requested_art.description
            .map(|f| markdown_to_html(&f, &comrak::Options::default()));

        let comments_with_sanitized_contents = requested_art.comments
            .into_iter().map(|comment| {
                // TODO: SANITIZE!!
                Comment {
                    contents: markdown_to_html(&comment.contents, &comrak::Options::default()),
                    ..comment
                }
            })
            .collect();

        Ok(template_to_response(ArtPage {
            user,
            original_uri,
            user_search_params: &query_params,

            user_can_edit_page,

            title: requested_art.base_art.title,
            artists: requested_art.base_art.creators,
            formatted_creation_date: requested_art.creation_date.to_string(),
            art_urls,
            tags: requested_art.tags,
            description: markdownified_description,

            comments: comments_with_sanitized_contents,

            older_art_url,
            newer_art_url,
        }))
    } else {
        Err(RootErrors::NotFound(original_uri, cookie_jar, user))
    }
}

/// Given an art piece's slug and any search parameters, returns the previous and next art pieces, relative to it.
/// The first value is the older one, the second is the newer one.
async fn get_older_and_newer_art_slugs(
    slug: &str,
    params: &structs::ArtSearchParameters,
    db_connection: &Object<Manager>,
) -> (Option<String>, Option<String>) {
    let mut sql_params: Vec<&(dyn ToSql + Sync)> = vec![&slug];

    // This query uses LAG and LEAD to get the previous and next page slugs, ordered by creation date first, and the page slug second.
    let query = format!(
        r#"SELECT
            previous_slug,
            next_slug
        FROM (
            SELECT
                page_slug,
                LAG(page_slug) OVER (ORDER BY creation_date, page_slug) AS previous_slug,
                LEAD(page_slug) OVER (ORDER BY creation_date, page_slug) AS next_slug
            FROM art
            {}
        ) AS pages_with_navigation
        WHERE page_slug = $1;"#,
        params.get_postgres_where(&mut sql_params)
    );

    let db_response = db_connection.query_one(&query, &sql_params).await;

    if let Ok(valid_response) = db_response {
        (
            valid_response.get("previous_slug"),
            valid_response.get("next_slug"),
        )
    } else {
        let err = db_response.unwrap_err();

        // If we get "unexpected number of rows", that's fine, it means the search was too specific and we got nothing.
        // Because of the UNIQUE constraint on page_slug, there's no way it's more than one response.
        if !err.to_string().contains("number of rows") {
            println!("DB Errored!\nQuery={}\nErr={:?}", query, err);
        }

        (None, None)
    }
}

/// Handle a user requesting to delete the page.
pub async fn delete_art_page(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        // If the user isn't logged in, kick them out.
        None => return Err(RootErrors::Unauthorized),
        Some(user) => user,
    };

    let requested_art = match structs::PageArt::get_by_slug(&db_connection, &art_slug).await {
        // If the requested art doesn't exist, also kick them out.
        None => {
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                Some(requesting_user),
            ))
        }
        Some(art) => art,
    };

    // If the user cant modify this art... you get the idea.
    if !requested_art.can_be_modified_by(&requesting_user) {
        return Err(RootErrors::Forbidden);
    }

    // The request is valid? Lovely! Let's start nuking stuff. First of all, take aim at the S3 bucket.
    let s3_client = state.s3_client.clone();

    // Get all of the art, and the thumbnail.
    let mut files_to_delete = requested_art.art_keys.clone();
    files_to_delete.push(requested_art.base_art.thumbnail_key.clone());

    crate::utils::delete_keys_from_s3(&s3_client, &state.config.s3_public_bucket, &files_to_delete)
        .await
        .map_err(|err|
            {
                eprintln!("[DELETE ART] When trying to delete artwork ID {}, name \"{}\", sending DELETE OBJECTS to S3 failed: {}", &requested_art.base_art.id, &requested_art.base_art.title, err);
                RootErrors::InternalServerError
            }
        )?;

    // Now that everything else is complete, nuke the page from the DB.
    const DELETION_QUERY: &str = "DELETE FROM art WHERE id=$1";
    db_connection
        .execute(DELETION_QUERY, &[&requested_art.base_art.id])
        .await
        .unwrap();

    // Yay! The page is deleted! :)
    let mut not_found_but_204 =
        RootErrors::NotFound(original_uri, cookie_jar, Some(requesting_user)).into_response();
    *not_found_but_204.status_mut() = axum::http::StatusCode::NO_CONTENT;
    Ok(not_found_but_204)
}
