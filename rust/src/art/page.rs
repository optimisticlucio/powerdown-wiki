use axum::{extract::{OriginalUri, Path, Query, State}, response::IntoResponse};
use http::Uri;
use postgres_types::{FromSql, ToSql, Type};
use askama::Template;
use crate::{errs::RootErrors, user::User, ServerState, utils::template_to_response};
use super::structs;
use deadpool::managed::Object;
use deadpool_postgres::Manager;

#[derive(Template)] 
#[template(path = "art/page.html")]
struct ArtPage<'a> {
    user: Option<User>,
    original_uri: Uri,
    user_search_params: &'a structs::ArtSearchParameters,

    title: String,
    artists: Vec<String>,
    formatted_creation_date: String,
    art_urls: Vec<String>,
    tags: Vec<String>,
    description: Option<String>, // Assumed to be markdown.

    newer_art_url: Option<String>,
    older_art_url: Option<String>
}

impl<'a> ArtPage<'a> {
    /// Given a URL, returns true if it's one that should be wrapped in a <video> tag.
    fn url_is_of_video(&self, url: &&String) -> bool {
        ["mov", "mp4", "avi"].iter().any(|ext| url.ends_with(ext))
    }
}

pub async fn art_page(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>,
    Query(query_params): Query<structs::ArtSearchParameters>,
    OriginalUri(original_uri): OriginalUri,
) -> impl IntoResponse {
    if let Some(requested_art) = structs::PageArt::get_by_slug(state.db_pool.get().await.unwrap(), &art_slug).await {
        let (older_art_url, newer_art_url) = get_older_and_newer_art_slugs(&art_slug, &query_params, state.db_pool.get().await.unwrap()).await;

        template_to_response(
            ArtPage {
                user: None, // TODO: Connect this to user system.
                original_uri,
                user_search_params: &query_params,

                title: requested_art.base_art.title,
                artists: requested_art.base_art.creators,
                formatted_creation_date: requested_art.creation_date.to_string(),
                art_urls: requested_art.art_urls,
                tags: requested_art.tags,
                description: requested_art.description,

                older_art_url,
                newer_art_url,
            }
        )
    }   
    else {
        RootErrors::NOT_FOUND.into_response()
    }
}

/// Given an art piece's slug and any search parameters, returns the previous and next art pieces, relative to it.
/// The first value is the older one, the second is the newer one.
async fn get_older_and_newer_art_slugs(slug: &str, params: &structs::ArtSearchParameters, db_connection: Object<Manager>) -> (Option<String>, Option<String>) {
    let mut sql_params: Vec<&(dyn ToSql + Sync)>= vec![&slug];

    // This query uses LAG and LEAD to get the previous and next page slugs, ordered by creation date first, and the page slug second.
    let query = 
    format!(
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
        params.get_postgres_where(&mut sql_params));

    let db_response = db_connection.query_one(&query, &sql_params).await;

    if let Ok(valid_response) = db_response {
        (valid_response.get("previous_slug"), valid_response.get("next_slug"))
    } else {
        let err = db_response.unwrap_err();

        // If we get "unexpected number of rows", that's fine, it means the search was too specific and we got nothing.
        // Because of the UNIQUE constraint on page_slug, there's no way it's more than one response.
        if !err.to_string().contains("number of rows") {
            println!("DB Errored!\nQuery={}\nErr={}", query, err.to_string());
        }
        
        (None, None)
    }
}