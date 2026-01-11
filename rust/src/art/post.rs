use super::structs::{ArtState, BaseArt};
use crate::art::structs::PageArt;
use crate::user::{User, UsermadePost};
use crate::utils::{self, template_to_response};
use crate::{errs::RootErrors, ServerState};
use askama::Template;
use aws_sdk_s3::presigning::PresigningConfig;
use axum::extract::{OriginalUri, Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{http, Json};
use http::Uri;
use rand::distr::SampleString;
use rand::{distr::Alphanumeric, Rng};
use serde::{self, Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinSet;
use url::Url;

/// Post Request Handler for art category.
#[axum::debug_handler]
pub async fn add_art(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_step): Json<ArtPostingSteps>,
) -> Result<Response, RootErrors> {
    match posting_step {
        ArtPostingSteps::RequestPresignedURLs { art_amount } => {
            give_user_presigned_s3_urls(art_amount, true, original_uri, cookie_jar, &state).await
        }
        ArtPostingSteps::UploadMetadata(page_art) => {
            // First let's make sure what we were given is even logical
            if let Err(err_explanation) = validate_recieved_page_art(&page_art) {
                return Err(RootErrors::BAD_REQUEST(
                    original_uri,
                    cookie_jar,
                    err_explanation,
                ));
            }

            // Makes sense? Good. Our job now.
            let db_connection = state
                .db_pool
                .get()
                .await
                .map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

            // Let's build the query.
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            columns.push("post_state".into());
            values.push(&ArtState::Processing);

            columns.push("page_slug".into());
            values.push(&page_art.base_art.slug);

            columns.push("creation_date".into());
            values.push(&page_art.creation_date);

            columns.push("title".into());
            values.push(&page_art.base_art.title);

            columns.push("creators".into());
            values.push(&page_art.base_art.creators);

            // TODO: RESIZE THUMBNAIL
            columns.push("thumbnail".into());

            let temp_thumbnail_key = match Url::parse(&page_art.base_art.thumbnail_key) {
                Err(err) => {
                    return Err(RootErrors::BAD_REQUEST(
                        original_uri,
                        cookie_jar,
                        format!(
                            "Invalid Thumbnail Url: {}",
                            &page_art.base_art.thumbnail_key
                        ),
                    ))
                }
                Ok(parsed_thumbnail_url) => parsed_thumbnail_url
                    .path()
                    .trim_start_matches("/")
                    .trim_start_matches(&state.config.s3_public_bucket)
                    .trim_start_matches("/")
                    .to_owned(),
            };

            values.push(&temp_thumbnail_key);

            columns.push("tags".into());
            values.push(&page_art.tags);

            columns.push("is_nsfw".into());
            values.push(&page_art.base_art.is_nsfw);

            let uploading_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

            if uploading_user.is_some() {
                columns.push("uploading_user_id".into());
                values.push(&uploading_user.as_ref().unwrap().id);
            }

            if let Some(description) = &page_art.description {
                // TODO: SANITIZE
                columns.push("description".into());
                values.push(description);
            }

            // Safe bc we're not inserting anything the user did. Everything user-inputted is passed as values later.
            let query = format!(
                "INSERT INTO art ({}) VALUES ({}) RETURNING id;",
                columns.join(","),
                (1..values.len() + 1)
                    .map(|i| format!("${i}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );

            let art_id: i32 = db_connection
                .query_one(&query, &values)
                .await
                .map_err(|err| {
                    eprintln!("[ART UPLOAD] Initial DB upload failed! {}", err.to_string());
                    RootErrors::INTERNAL_SERVER_ERROR
                })?
                .get(0);

            // ---- We have the ID? process the thumbnail and update. ----
            let target_s3_folder = format!("art/{art_id}");

            let target_thumbnail_key = format!("{target_s3_folder}/thumbnail");

            utils::move_temp_s3_file(
                state.s3_client.clone(),
                &state.config,
                &temp_thumbnail_key,
                &state.config.s3_public_bucket,
                &target_thumbnail_key,
            )
            .await
            .map_err(|err| {
                eprintln!(
                    "[ART UPLOAD] Converting thumbnail of art {art_id} failed, {}",
                    err.to_string()
                );
                RootErrors::INTERNAL_SERVER_ERROR
            })?;

            db_connection
                .execute(
                    "UPDATE art SET thumbnail=$1 WHERE id=$2",
                    &[&target_thumbnail_key, &art_id],
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Updating thumbnail key in DB of art {art_id} failed, {}",
                        err.to_string()
                    );
                    RootErrors::INTERNAL_SERVER_ERROR
                })?;

            // ---- Now that the main art file is up, upload the individual art pieces. ----
            let query = "INSERT INTO art_file (belongs_to,internal_order,s3_key) VALUES ($1,$2,$3)";

            let mut art_upload_tasks = JoinSet::new();

            // The names of the files we're supposed to create, incase the upload fails.
            let temp_file_keys = page_art
                .art_keys
                .iter()
                .map(|given_art_key| {
                    Ok(Url::parse(&given_art_key)
                        .map_err(|_| format!("Invalid Art Url: {}", &given_art_key))?
                        .path()
                        .trim_start_matches("/")
                        .trim_start_matches(&state.config.s3_public_bucket)
                        .trim_start_matches("/")
                        .to_owned())
                })
                .collect::<Result<Vec<String>, String>>();

            // Doing this so the compiler doesn't whine about ownership re: the error. If you have a better way, please do that.
            let temp_file_keys = match temp_file_keys {
                Err(err_string) => {
                    return Err(RootErrors::BAD_REQUEST(
                        original_uri,
                        cookie_jar,
                        err_string,
                    ))
                }
                Ok(temp_keys) => temp_keys,
            };

            for (s3_key, index) in temp_file_keys.iter().zip(1i32..) {
                // Clone everything to move it into the async move.
                let s3_key = s3_key.clone();
                let index = index.clone();
                let art_id = art_id.clone();
                let s3_client = state.s3_client.clone();
                let public_bucket_key = state.config.s3_public_bucket.clone();
                let config = state.config.clone();
                let db_connection = state
                    .db_pool
                    .get()
                    .await
                    .map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;
                let target_s3_folder = target_s3_folder.clone();

                // tokio::spawn lets all the tasks run simultaneously, which is nice.
                art_upload_tasks.spawn(async move {
                    let file_key = format!(
                        "{target_s3_folder}/{}",
                        s3_key.split_terminator("/").last().unwrap()
                    );

                    utils::move_temp_s3_file(
                        s3_client,
                        &config,
                        &s3_key,
                        &public_bucket_key,
                        &file_key,
                    )
                    .await
                    .map_err(|err| err.to_string())?;

                    let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
                    values.push(&art_id);
                    values.push(&index);
                    values.push(&file_key);

                    db_connection
                        .execute(query, &values)
                        .await
                        .map_err(|err| err.to_string())
                });
            }

            // Now collect everything that ran async, make sure nothing fucked up.
            let art_upload_results = art_upload_tasks.join_all().await;

            let failed_uploads: Vec<_> = art_upload_results
                .into_iter()
                .filter_map(|result| result.err())
                .collect();

            if !failed_uploads.is_empty() {
                let file_keys_to_delete: Vec<String> = temp_file_keys
                    .iter()
                    .map(|key| {
                        format!(
                            "{target_s3_folder}/{}",
                            key.split_terminator("/").last().unwrap()
                        )
                    })
                    .collect();

                // TODO: DELETE ANY OF THE SUCCESSFULLY PROCESSED FILES.

                let _ = db_connection.execute("DELETE FROM art WHERE id=$1", &[&art_id]);

                eprintln!(
                    "[ART POST] Failed to move files from temp to permanent location! [{}]",
                    failed_uploads.join(", ")
                );

                return Err(RootErrors::INTERNAL_SERVER_ERROR);
            }

            // ---- Now that we finished, set the appropriate art state. ----

            db_connection
                .execute(
                    "UPDATE art SET post_state=$1 WHERE id=$2",
                    &[&ArtState::Public, &art_id],
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Setting post state of id {art_id} to public failed?? {}",
                        err.to_string()
                    );
                    RootErrors::INTERNAL_SERVER_ERROR
                })?;

            Ok(Redirect::to(&format!("/art/{}", page_art.base_art.slug)).into_response())
        }
        _ => Err(RootErrors::BAD_REQUEST(
            original_uri,
            cookie_jar,
            "invalid upload step".to_string(),
        )),
    }
}

/// Struct for reading the "steps" that a user (well, their client) needs to take to successfully upload art to the site.
#[derive(Debug, Deserialize)]
#[serde(tag = "step")]
pub enum ArtPostingSteps {
    #[serde(rename = "1")]
    RequestPresignedURLs {
        art_amount: u8, // It shouldn't be any bigger than *25* and positive. even u8 is overkill.
    },

    #[serde(rename = "2")]
    UploadMetadata(PageArt),
}

#[derive(Debug, Serialize)]
struct PresignedUrlsResponse {
    thumbnail_presigned_url: Option<String>,
    art_presigned_urls: Vec<String>,
}

#[derive(Debug, Template)]
#[template(path = "art/new.html")]
struct ArtPostingPage {
    user: Option<User>,
    original_uri: Uri,
}

pub async fn art_posting_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    Ok(template_to_response(ArtPostingPage {
        user: User::easy_get_from_cookie_jar(&state, &cookie_jar).await?,
        original_uri,
    }))
}

pub async fn edit_art_put_request(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_step): Json<ArtPostingSteps>,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

    let existing_art = match PageArt::get_by_slug(&db_connection, &art_slug).await {
        None => return Err(RootErrors::NOT_FOUND(original_uri, cookie_jar)),
        Some(existing_art) => existing_art,
    };

    // Who's asking to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        None => return Err(RootErrors::UNAUTHORIZED),
        Some(requesting_user) => requesting_user,
    };

    // If they don't have permissions to do this, shoot back HTTP 403.
    if !(existing_art.can_be_modified_by(&requesting_user)) {
        return Err(RootErrors::FORBIDDEN);
    }

    match posting_step {
        ArtPostingSteps::RequestPresignedURLs { art_amount } => {
            give_user_presigned_s3_urls(art_amount, true, original_uri, cookie_jar, &state).await
        }
        ArtPostingSteps::UploadMetadata(mut sent_page_art) => {
            // First let's make sure what we were given is even logical
            if let Err(err_explanation) = validate_recieved_page_art(&sent_page_art) {
                return Err(RootErrors::BAD_REQUEST(
                    original_uri,
                    cookie_jar,
                    err_explanation,
                ));
            }

            // TODO - MOVE TEMP ART SENT OVER TO PERMANENT STORAGE

            // Now that everything is uploaded properly, let's start modifying what needs to be changed.
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            columns.push("post_state".into());
            values.push(&ArtState::Processing);

            if sent_page_art.base_art.slug != existing_art.base_art.slug {
                columns.push("page_slug".into());
                values.push(&sent_page_art.base_art.slug);
            }

            if sent_page_art.creation_date != existing_art.creation_date {
                columns.push("creation_date".into());
                values.push(&sent_page_art.creation_date);
            }

            if sent_page_art.base_art.title != existing_art.base_art.title {
                columns.push("title".into());
                values.push(&sent_page_art.base_art.title);
            }

            if sent_page_art.base_art.creators != existing_art.base_art.creators {
                columns.push("creators".into());
                values.push(&sent_page_art.base_art.creators);
            }

            if sent_page_art.tags != existing_art.tags {
                columns.push("tags".into());
                values.push(&sent_page_art.tags);
            }

            if sent_page_art.base_art.is_nsfw != existing_art.base_art.is_nsfw {
                columns.push("is_nsfw".into());
                values.push(&sent_page_art.base_art.is_nsfw);
            }

            if sent_page_art.base_art.thumbnail_key != existing_art.base_art.thumbnail_key {
                columns.push("thumbnail".into());
                values.push(&sent_page_art.base_art.thumbnail_key);
            }

            if sent_page_art.description != existing_art.description {
                columns.push("description".into());
                values.push(&sent_page_art.description);
            }

            // Safe bc nothing user-written is passed into the string. User values are in `values`
            let query = format!(
                "UPDATE art ({}) VALUES ({}) WHERE id={};",
                columns.join(","),
                (1..values.len() + 1)
                    .map(|i| format!("${i}"))
                    .collect::<Vec<_>>()
                    .join(","),
                format!("${}", values.len() + 2)
            );

            values.push(&existing_art.base_art.id);
            db_connection
                .execute(&query, &values)
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Updating metadata of art id {}, named \"{}\", failed. {}",
                        &existing_art.base_art.id,
                        &existing_art.base_art.title,
                        err.to_string()
                    );
                    RootErrors::INTERNAL_SERVER_ERROR
                })?;

            // TODO - DELETE ANY REMOVED ART, AND MOVE THE NEW ART INTO PLACE

            // ---- Now that we finished, set the appropriate art state. ----

            db_connection
                .execute(
                    "UPDATE art SET post_state=$1 WHERE id=$2",
                    &[&ArtState::Public, &existing_art.base_art.id],
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Setting post state of id {} to public failed?? {}",
                        existing_art.base_art.id,
                        err.to_string()
                    );
                    RootErrors::INTERNAL_SERVER_ERROR
                })?;

            Ok(Redirect::to(&format!("/art/{}", sent_page_art.base_art.slug)).into_response())
        }
    }
}

/// Given an amount of urls requested by the user, sends the user back the appropriate amount of new temp S3 presigned URLs. May also request an extra url for the thumbnail.
async fn give_user_presigned_s3_urls(
    requested_amount_of_urls: u8,
    including_thumbnail: bool,
    original_uri: Uri,
    cookie_jar: tower_cookies::Cookies,
    state: &ServerState,
) -> Result<Response, RootErrors> {
    if requested_amount_of_urls < 1 {
        Err(RootErrors::BAD_REQUEST(
            original_uri,
            cookie_jar,
            "art post must have at least one art piece".to_string(),
        ))
    } else if requested_amount_of_urls > 25 {
        Err(RootErrors::BAD_REQUEST(
            original_uri,
            cookie_jar,
            "for the good of mankind, don't put that many art pieces in one post. split them up"
                .to_string(),
        ))
    } else {
        let amount_of_presigned_urls_needed =
            requested_amount_of_urls + if including_thumbnail { 1 } else { 0 }; // The art, plus the thumbnail.

        let temp_key_tasks: Vec<_> = (0..amount_of_presigned_urls_needed)
            .map(|_| {
                let s3_client = state.s3_client.clone();
                let public_bucket_key = state.config.s3_public_bucket.clone();

                tokio::spawn(async move {
                    let random_key = Alphanumeric.sample_string(&mut rand::rng(), 64);
                    let temp_art_key = format!("temp/art/{}", random_key);

                    // get s3 to open a presigned URL for the temp key.
                    s3_client
                        .put_object()
                        .bucket(public_bucket_key)
                        .key(temp_art_key)
                        .presigned(
                            PresigningConfig::expires_in(Duration::from_secs(300)).unwrap(), // Five minutes to upload. May be too much?
                        )
                        .await
                        .map(|x| x.uri().to_string())
                })
            })
            .collect();

        let mut temp_keys_for_presigned = Vec::new();

        for task in temp_key_tasks {
            let uri = task
                .await
                .map_err(|err| {
                    eprintln!("[ART POST STAGE 1] Tokio Join Err! {}", err.to_string());
                    RootErrors::INTERNAL_SERVER_ERROR
                })?
                .map_err(|err| {
                    eprintln!(
                        "[ART POST STAGE 1] SDK presigned URL creation err! {}",
                        err.to_string()
                    );
                    RootErrors::INTERNAL_SERVER_ERROR
                })?;

            temp_keys_for_presigned.push(uri);
        }

        // Send back the urls as a json.
        let response = serde_json::to_string(&PresignedUrlsResponse {
            thumbnail_presigned_url: if including_thumbnail {
                temp_keys_for_presigned.pop()
            } else {
                None
            },
            art_presigned_urls: temp_keys_for_presigned,
        })
        .unwrap();

        Ok(response.into_response())
    }
}

/// Given a user-created Page Art, validates that it makes sense. If it doesn't, returns a readable explanation why.
fn validate_recieved_page_art(recieved_page_art: &PageArt) -> Result<(), String> {
    if recieved_page_art.art_keys.is_empty() {
        return Err("No Art Keys Given".to_owned());
    }

    // TODO: Validate all of the given values make sense.

    Ok(())
}
