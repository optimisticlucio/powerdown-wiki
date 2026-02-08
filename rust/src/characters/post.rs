use crate::characters::BaseCharacter;
use crate::user::User;
use crate::utils::sql::PostState;
use crate::utils::{self, PostingSteps, PresignedUrlsResponse, get_temp_s3_presigned_urls, template_to_response};
use crate::{
    characters::structs::{PageCharacter},
    errs::RootErrors,
    ServerState,
};
use axum::extract::{OriginalUri, State};
use axum::{http, Json};
use axum::response::{IntoResponse, Redirect, Response};
use http::Uri;
use askama::Template;

const CHARACTER_THUMBNAIL_COMPRESSION_SETTINGS: utils::file_compression::LossyCompressionSettings = utils::file_compression::LossyCompressionSettings {
                            max_width: Some(100),
                            max_height: Some(100),
                            quality: 85
                        };

const CHARACTER_IMAGE_COMPRESSION_SETTINGS: utils::file_compression::LossyCompressionSettings = utils::file_compression::LossyCompressionSettings {
                            max_width: Some(500),
                            max_height: Some(500),
                            quality: 90
                        };

const CHARACTER_LOGO_COMPRESSION_SETTINGS: utils::file_compression::LossyCompressionSettings = utils::file_compression::LossyCompressionSettings {
                            max_width: Some(100),
                            max_height: Some(100),
                            quality: 90
                        };

#[axum::debug_handler]
pub async fn add_character(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_step): Json<PostingSteps<PageCharacter>>
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    // Who's trying to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        Some(user) => user,
        None => return Err(RootErrors::Unauthorized)
    };

    if !requesting_user.user_type.permissions().can_post_characters {
        return Err(RootErrors::Forbidden);
    }

    match posting_step {
        PostingSteps::RequestPresignedURLs { file_amount } => {
            // TODO - Ensure it's an amount of art that makes sense. Right now the only variable is whether a page has a logo in it.

            let presigned_urls = get_temp_s3_presigned_urls(&state, file_amount.into(), "characters")
                .await
                .map_err(|err| {
                    eprintln!("[POSTING CHARACTER] Failed to get presigned URLs! {}", err);
                    RootErrors::InternalServerError
                })?;

            // Now return the presigned urls as a json
            Ok(
                serde_json::to_string(
                    &PresignedUrlsResponse{
                        presigned_urls
                    }
                ).unwrap().into_response())
        }
        PostingSteps::UploadMetadata(mut recieved_page_character) => {
            sanitize_recieved_page_character(&mut recieved_page_character, &state);

            if let Err(err_string) =  validate_recieved_page_character(&recieved_page_character) {
                return Err(RootErrors::BadRequest(err_string));
            }

            // Check if this character already exists. If it does, throw an error.
            if BaseCharacter::get_by_slug(&db_connection, &recieved_page_character.base_character.slug).await.is_some() {
                return Err(RootErrors::BadRequest(
                    format!("The slug {} already exists.", &recieved_page_character.base_character.slug)
                ));
            }

            // Let's build our query.
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            columns.push("page_slug".into());
            values.push(&recieved_page_character.base_character.slug);

            columns.push("short_name".into());
            values.push(&recieved_page_character.base_character.name);

            columns.push("subtitles".into());
            values.push(&recieved_page_character.subtitles);

            columns.push("creator".into());
            values.push(&recieved_page_character.creator);

            columns.push("infobox".into());
            values.push(&recieved_page_character.infobox);

            if let Some(overlay_css) = &recieved_page_character.overlay_css {
                // TODO: SANITIZE
                columns.push("overlay_css".into());
                values.push(overlay_css);
            }

            if let Some(page_text) = &recieved_page_character.page_contents {
                // TODO: SANITIZE
                columns.push("page_text".into());
                values.push(page_text);
            }

            columns.push("is_hidden".into());
            values.push(&recieved_page_character.base_character.is_hidden);

            if let Some(retirement_reason) = &recieved_page_character.retirement_reason {
                columns.push("retirement_reason".into());
                values.push(retirement_reason);
            }

            if let Some(long_name) = &recieved_page_character.long_name {
                columns.push("long_name".into());
                values.push(long_name);
            }

            if let Some(logo) = &recieved_page_character.logo_url {
                columns.push("logo".into());
                values.push(logo);
            }

            if let Some(birthday) = &recieved_page_character.base_character.birthday {
                columns.push("birthday".into());
                values.push(birthday);
            }

            if let Some(tag) = &recieved_page_character.tag {
                columns.push("relevant_tag".into());
                values.push(tag);
            }

            // We don't have these values *yet*, but there's a db constraint, so just shove garbage in there.
            columns.push("thumbnail".into());
            values.push(&"missing_thumbnail");

            columns.push("page_image".into());
            values.push(&"missing_img");

            // So people don't see the partially-processed page.
            columns.push("post_state".into());
            values.push(&PostState::Processing);

            // Now we have the character! Well, most pieces of the character. Let's get their ID.
            // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
            let creation_query = format!(
                "INSERT INTO character ({}) VALUES ({}) RETURNING id;",
                columns.join(","),
                (1..values.len() + 1)
                    .map(|i| format!("${i}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );

            let character_id: i32 = db_connection.query_one(&creation_query, &values)
                .await
                .map_err(|err| {
                    eprintln!("[CHARACTER POSTING] Character {} failed initial creation! {:?}", &recieved_page_character.base_character.name, err);
                    RootErrors::InternalServerError
                })?
                .get(0);

            // The character is on the DB! Well, most of them. Now let's move their art to its final location and put it in.
            let target_s3_folder = format!("characters/{character_id}");
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
            let s3_client = state.s3_client.clone();
            
            // Move thumbnail.
            let thumbnail_target_s3_key = format!("{target_s3_folder}/thumbnail");
            let thumbnail_s3_key = utils::move_and_lossily_compress_temp_s3_img(
                    &s3_client,
                    &state.config,
                    &recieved_page_character.base_character.thumbnail_key,
                    &state.config.s3_public_bucket,
                    &thumbnail_target_s3_key,
                    Some(CHARACTER_THUMBNAIL_COMPRESSION_SETTINGS)
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[CHARACTER POSTING] Converting thumbnail of character {character_id} failed, {:?}",
                        err
                    );

                    // Delete the processing character before returning error.
                    let _ = db_connection.execute("DELETE FROM character WHERE id=$1", &[&character_id]);

                    RootErrors::InternalServerError
                })?;
            
            columns.push("thumbnail".into());
            values.push(&thumbnail_s3_key);
            
            // Move main page art
            let main_art_target_s3_key = format!("{target_s3_folder}/page_art");
            let main_art_key = utils::move_and_lossily_compress_temp_s3_img(
                    &s3_client,
                    &state.config,
                    &recieved_page_character.page_img_key,
                    &state.config.s3_public_bucket,
                    &main_art_target_s3_key,
                    Some(CHARACTER_IMAGE_COMPRESSION_SETTINGS)
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[CHARACTER POSTING] Converting main art of character {character_id} failed, {:?}",
                        err
                    );

                    // Delete the processing character before returning error.
                    let _ = db_connection.execute("DELETE FROM character WHERE id=$1", &[&character_id]);

                    RootErrors::InternalServerError
                })?;

            columns.push("page_image".into());
            values.push(&main_art_key);

            let compressed_logo_key: String;
            if let Some(temp_logo_key) = &recieved_page_character.logo_url {
                let target_key = format!("{target_s3_folder}/logo");
                compressed_logo_key = utils::move_and_lossily_compress_temp_s3_img(
                        &s3_client,
                        &state.config,
                        &temp_logo_key,
                        &state.config.s3_public_bucket,
                        &target_key,
                        Some(CHARACTER_LOGO_COMPRESSION_SETTINGS)
                    )
                    .await
                    .map_err(|err| {
                        eprintln!(
                            "[CHARACTER POSTING] Converting logo of character {character_id} failed, {:?}",
                            err
                        );

                        // Delete the processing character before returning error.
                        let _ = db_connection.execute("DELETE FROM character WHERE id=$1", &[&character_id]);

                        RootErrors::InternalServerError
                    })?;
                
                columns.push("logo".into());
                values.push(&compressed_logo_key);
            }

            // This is the final push!
            columns.push("post_state".into());
            values.push(&PostState::Public);

            // Update the image values of the character
            // SAFETY: No user-passed values are in the query, they're all in `values`
            let query = format!(
                "UPDATE character SET {} WHERE id={};",
                columns.iter().enumerate()
                    .map(|(index, value)| format!("{}=${}", value, index+1))
                    .collect::<Vec<_>>()
                    .join(","),
                format!("${}", columns.len() + 1)
            );

            values.push(&character_id);

            db_connection
                .execute(&query, &values)
                .await
                .map_err(|err| {
                    println!(
                        "[CHARACTER POSTING] Error in db query execution!\nQuery: {}\nError: {:?}",
                        query, err
                    );

                    // Delete the processing character before returning error.
                    let _ = db_connection.execute("DELETE FROM character WHERE id=$1", &[&character_id]);

                    RootErrors::InternalServerError
                })?;

            Ok(
                Redirect::to(&format!(
                    "/characters/{}",
                    &recieved_page_character.base_character.slug
                )).into_response()
            )
        }
    }
}

#[derive(Debug, Template)]
#[template(path = "characters/new.html")]
pub struct CharacterPostingPage {
    pub user: Option<User>,
    pub original_uri: Uri,

    /// Incase we're editing an existing page, pass the character here.
    pub character_being_modified: Option<PageCharacter>,

    /// The URL to which our upload button will be talking to. If empty, messages the current URI.
    pub target_button_url: Option<String>,
}

pub async fn character_posting_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    Ok(template_to_response(CharacterPostingPage {
        user: User::easy_get_from_cookie_jar(&state, &cookie_jar).await?,
        original_uri,

        character_being_modified: None,
        target_button_url: None
    }))
}

/// Given a user-created Page Art, validates that it makes sense. If it doesn't, returns a readable explanation why.
fn validate_recieved_page_character(recieved_page_character: &PageCharacter) -> Result<(), String> {
    if !utils::is_valid_slug(&recieved_page_character.base_character.slug) {
        return Err("Given invalid slug. Slugs must be made of either lowercase letters or numbers, and may include hyphens or underscores in the middle.".to_string());
    }

    // TODO - Validate

    Ok(())
}

/// Given a Page Character, cleans up any invalid or nonsensical values, such as empty strings in lists.
/// NOTE: Does not make sure the values make _logical_ sense, only that we don't deal with trivially incorrect data.
fn sanitize_recieved_page_character(recieved_page_character: &mut PageCharacter, state: &ServerState) {
    // Clean the keys given by the user.
    recieved_page_character.logo_url = match &recieved_page_character.logo_url {
        None => None,
        Some(given_logo) => utils::clean_passed_key(given_logo, state)
    };
    recieved_page_character.page_img_key = utils::clean_passed_key(&recieved_page_character.page_img_key, state).unwrap_or_default();
    recieved_page_character.base_character.thumbnail_key = utils::clean_passed_key(
        &recieved_page_character.base_character.thumbnail_key,
        state
    ).unwrap_or_default();

    // Make sure none of the lists have empty values in them
    recieved_page_character.subtitles = recieved_page_character.subtitles
        .iter()
        .filter_map(|subtitle| if subtitle.is_empty() { None } else { Some(subtitle.to_string()) })
        .collect();

    // Make sure none of the Options have empty values in them.
    recieved_page_character.custom_css = recieved_page_character.custom_css
        .as_deref()
        .filter(|custom_css| custom_css.is_empty())
        .map(|s| s.to_string());
    
    // TODO - Finish Sanitization
}