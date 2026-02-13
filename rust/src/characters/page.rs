use crate::user::UsermadePost;
use crate::utils::template_to_response;
use crate::{characters::structs::PageCharacter, errs::RootErrors, user::User, ServerState};
use askama::Template;
use axum::response::IntoResponse;
use axum::{
    extract::{OriginalUri, Path, State},
    response::Response,
};
use comrak::markdown_to_html;
use http::Uri;
use rand::seq::IndexedRandom;

#[derive(Debug, Template)]
#[template(path = "characters/page.html")]
struct CharacterPage<'a> {
    user: Option<User>,
    original_uri: Uri,

    character: PageCharacter,

    name: String,

    retirement_reason: Option<&'a str>,

    subtitle: &'a str,

    content: Option<&'a str>,
}

pub async fn character_page(
    Path(character_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();
    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    if let Some(chosen_char) = PageCharacter::get_by_slug(&db_connection, &character_slug).await {
        let parsed_content = chosen_char.page_contents.as_ref().map(|contents| {
            parse_character_page_contents(contents).unwrap_or("PARSING FAILED!".to_owned())
        });

        let retirement_reason = chosen_char
            .retirement_reason
            .as_ref()
            .map(|f| markdown_to_html(f, &comrak::Options::default()));

        let random_subtitle = chosen_char
            .subtitles
            .choose(&mut rand::rng())
            .unwrap()
            .clone();

        Ok(template_to_response(CharacterPage {
            user: requesting_user,
            original_uri,

            retirement_reason: retirement_reason.as_deref(),

            name: chosen_char
                .long_name
                .clone()
                .unwrap_or(chosen_char.base_character.name.clone()),

            subtitle: &random_subtitle,

            content: parsed_content.as_deref(),

            character: chosen_char,
        }))
    } else {
        Err(RootErrors::NotFound(
            original_uri,
            cookie_jar,
            requesting_user,
        ))
    }
}

fn parse_character_page_contents(unparsed_contents: &str) -> Option<String> {
    let mut parsed_contents = markdown_to_html(
        unparsed_contents,
        &comrak::Options {
            extension: comrak::ExtensionOptions {
                ..comrak::ExtensionOptions::default()
            },
            parse: comrak::ParseOptions {
                ..comrak::ParseOptions::default()
            },
            render: comrak::RenderOptions {
                ..comrak::RenderOptions::default()
            },
        },
    )
    .replace("<h1>", r#"</div> <h1> <span>"#)
    .replace("</h1>", r#"</span> </h1> <div class="text">"#);

    if let Some(first_added_div_exit_tag) = parsed_contents.find("</div>") {
        let text_exists_before_first_header = first_added_div_exit_tag > 0;

        if text_exists_before_first_header {
            parsed_contents.insert_str(0, r#"<div class="text">"#);
        } else {
            parsed_contents = parsed_contents.replacen("</div>", "", 1);
        }
    }

    Some(parsed_contents)
}

/// Handle a user requesting to delete the page.
pub async fn delete_character_page(
    Path(character_slug): Path<String>,
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

    let requested_character =
        match super::structs::PageCharacter::get_by_slug(&db_connection, &character_slug).await {
            // If the requested art doesn't exist, also kick them out.
            None => {
                return Err(RootErrors::NotFound(
                    original_uri,
                    cookie_jar,
                    Some(requesting_user),
                ))
            }
            Some(character) => character,
        };

    // If the user cant modify this art... you get the idea.
    if !requested_character.can_be_modified_by(&requesting_user) {
        return Err(RootErrors::Forbidden);
    }

    // The request is valid? Lovely! Let's start nuking stuff. First of all, take aim at the S3 bucket.
    let s3_client = state.s3_client.clone();

    // Get all of the image files.
    let mut files_to_delete = vec![
        requested_character.base_character.thumbnail_key,
        requested_character.page_img_key,
    ];
    if let Some(logo_url) = requested_character.logo_url {
        files_to_delete.push(logo_url);
    }

    crate::utils::delete_keys_from_s3(&s3_client, &state.config.s3_public_bucket, &files_to_delete)
        .await
        .map_err(|err|
            {
                eprintln!("[DELETE CHARACTER] When trying to delete character ID {}, name \"{}\", sending DELETE OBJECTS to S3 failed: {}", &requested_character.base_character.db_id, &requested_character.base_character.name, err);
                RootErrors::InternalServerError
            }
        )?;

    // Now that everything else is complete, nuke the page from the DB.
    const DELETION_QUERY: &str = "DELETE FROM character WHERE id=$1";
    db_connection
        .execute(DELETION_QUERY, &[&requested_character.base_character.db_id])
        .await
        .unwrap();

    println!(
        "[CHARACTER DELETION] User {} (ID:{}) DELETED character {} (ID:{}, SLUG:{})",
        requesting_user.display_name,
        requesting_user.id,
        requested_character.base_character.name,
        requested_character.base_character.db_id,
        requested_character.base_character.slug
    );

    // Yay! The page is deleted! :)
    let mut not_found_but_204 =
        RootErrors::NotFound(original_uri, cookie_jar, Some(requesting_user)).into_response();
    *not_found_but_204.status_mut() = axum::http::StatusCode::NO_CONTENT;
    Ok(not_found_but_204)
}
