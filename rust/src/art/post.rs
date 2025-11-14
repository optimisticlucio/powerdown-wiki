use std::time::Duration;
use askama::Template;
use aws_sdk_s3::presigning::PresigningConfig;
use axum::extract::{OriginalUri, State};
use axum::{Json, http};
use axum::response::{IntoResponse, Response, Redirect};
use http::Uri;
use rand::distr::SampleString;
use tokio::io::Join;
use crate::art::structs::PageArt;
use crate::user::User;
use crate::utils::{self, compress_image_lossless, get_s3_object_url, template_to_response, text_or_internal_err};
use crate::{ServerState, errs::RootErrors};
use super::{structs::{BaseArt}};
use rand::{distr::Alphanumeric, Rng};
use tokio::task::JoinSet;
use serde::{self, Deserialize, Serialize};

/// Post Request Handler for art category.
#[axum::debug_handler]
pub async fn add_art(State(state): State<ServerState>, Json(posting_step): Json<ArtPostingSteps>) -> Result<Response, RootErrors> {
    match posting_step {
        ArtPostingSteps::RequestPresignedURLs { art_amount } => {
            if art_amount < 1 {
                Err(RootErrors::BAD_REQUEST("art post must have at least one art piece".to_string()))
            } else if art_amount > 25 {
                Err(RootErrors::BAD_REQUEST("for the good of mankind, don't put that many art pieces in one post. split them up".to_string()))
            } else {
                let amount_of_presigned_urls_needed = art_amount + 1; // The art, plus the thumbnail.
                
                let temp_key_tasks: Vec<_> = (0..amount_of_presigned_urls_needed)
                    .map(|_| {
                        let s3_client = state.s3_client.clone();
                        let public_bucket_key = state.config.s3_public_bucket.clone();

                        tokio::spawn(async move {
                            let random_key = Alphanumeric.sample_string(&mut rand::rng(), 64);
                            let temp_art_key = format!("temp/art/{}", random_key);

                            // get s3 to open a presigned URL for the temp key.
                            s3_client.put_object()
                                .bucket(public_bucket_key)
                                .key(temp_art_key)
                                .presigned(
                                    PresigningConfig::expires_in(Duration::from_secs(300)).unwrap() // Five minutes to upload. May be too much?
                                )
                                .await
                                .map(|x| x.uri().to_string())
                        })
                        
                    })
                    .collect();
                    
                let mut temp_keys_for_presigned = Vec::new();

                for task in temp_key_tasks {
                    let uri = task.await
                        .map_err(|err| {
                            eprintln!("[ART POST STAGE 1] Tokio Join Err! {}", err.to_string());
                            RootErrors::INTERNAL_SERVER_ERROR
                        })?
                        .map_err(|err| {
                            eprintln!("[ART POST STAGE 1] SDK presigned URL creation err! {}", err.to_string());
                            RootErrors::INTERNAL_SERVER_ERROR
                        })?;

                    temp_keys_for_presigned.push(uri);
                }

                // Send back the urls as a json.
                let response = serde_json::to_string(&PresignedUrlsResponse {
                    thumbnail_presigned_url: temp_keys_for_presigned.pop().unwrap(),
                    art_presigned_urls: temp_keys_for_presigned
                }).unwrap();

                Ok(response.into_response())
            }
        },
        ArtPostingSteps::UploadMetadata(page_art) => {
            // TODO: Validate all of the given values make sense.
            
            let db_connection = state.db_pool.get().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

            // Let's build the query.
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            columns.push("post_state".into());
            values.push(&super::structs::ArtState::Public);

            columns.push("page_slug".into());
            values.push(&page_art.base_art.slug);

            columns.push("creation_date".into());
            values.push(&page_art.creation_date);

            columns.push("title".into());
            values.push(&page_art.base_art.title);

            columns.push("creators".into());
            values.push(&page_art.base_art.creators);

            // TODO: CONVERT GIVEN URL TO KEY
            // TODO: COMPRESS THUMBNAIL
            // TODO: RESIZE THUMBNAIL
            columns.push("thumbnail".into());
            values.push(&page_art.base_art.thumbnail_url); 

            columns.push("tags".into());
            values.push(&page_art.tags);

            columns.push("is_nsfw".into());
            values.push(&page_art.base_art.is_nsfw);

            if let Some(description) = &page_art.description {
                // TODO: SANITIZE
                columns.push("description".into());
                values.push(description);
            }

            let query = format!(
                "INSERT INTO art ({}) VALUES ({}) RETURNING id;",
                columns.join(","),
                (1..values.len()).map(|i| format!("${i}")).collect::<Vec<_>>().join(",")
            );

            let art_id: u32 = db_connection.query_one(&query, &values).await
                .map_err(|err| {
                    eprintln!("[ART UPLOAD] Initial DB upload failed! {}", err.to_string());
                    RootErrors::INTERNAL_SERVER_ERROR
                })?
                .get(0);

            // ---- Now that the main art file is up, upload the individual art pieces. ----
            let query = "INSERT INTO art_file (belongs_to,internal_order,s3_key) VALUES ($1,$2,$3)";
            
            let mut art_upload_tasks = JoinSet::new();

            for (url, index) in page_art.art_urls.iter().zip(1u32..) {
                // Clone everything to move it into the async move.
                let url = url.clone();
                let index = index.clone();
                let art_id = art_id.clone();
                let s3_client = state.s3_client.clone();
                let public_bucket_key = state.config.s3_public_bucket.clone();
                let db_connection = state.db_pool.get().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

                // tokio::spawn lets all the tasks run simultaneously, which is nice.
                art_upload_tasks.spawn(async move {
                        let temp_file_key = "TODO: GET TEMP FILE KEY FROM THE TEMP URL";
                        let file_key = "TODO: GET FILE KEY FROM THE TEMP URL";

                        // TODO: HANDLE ERR HERE
                        let _ = utils::move_temp_s3_file(s3_client, temp_file_key, &public_bucket_key, file_key).await;

                        let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
                        values.push(&art_id);
                        values.push(&index);
                        values.push(&url); 

                        db_connection.execute(query, &values).await
                    });
            }

            // Now collect everything that ran async, make sure nothing fucked up.
            let art_upload_results = art_upload_tasks.join_all().await;

            let failed_uploads: Vec<_> = art_upload_results.into_iter()
                .filter_map(|result| result.err())
                .collect();

            if !failed_uploads.is_empty() {
                // TODO: DELETE ANY OF THE SUCCESSFULLY PROCESSED FILES.

                let _ = db_connection.execute("DELETE FROM art WHERE id=$1", &[&art_id]);
                
                eprintln!("[ART POST] Failed to move files from temp to permanent location! {:?}",
                        failed_uploads);

                return Err(RootErrors::INTERNAL_SERVER_ERROR)
            }

            Ok(Redirect::to(&format!("/art/{}", page_art.base_art.slug)).into_response())
        },
        _ => Err(RootErrors::BAD_REQUEST("invalid upload step".to_string()))
    }
}


/// Struct for reading the "steps" that a user (well, their client) needs to take to successfully upload art to the site.
#[derive(Deserialize)]
#[serde(tag = "step")]
pub enum ArtPostingSteps {
    #[serde(rename = "1")]
    RequestPresignedURLs {
        art_amount: u8 // It shouldn't be any bigger than *25* and positive. even u8 is overkill.
    },

    #[serde(rename="2")] 
    UploadMetadata(PageArt),
}

#[derive(Serialize)]
struct PresignedUrlsResponse {
    thumbnail_presigned_url: String,
    art_presigned_urls: Vec<String>,
}

#[derive(Template)] 
#[template(path = "art/post.html")]
struct ArtPostingPage {
    user: Option<User>,
    original_uri: Uri,
}

pub async fn art_posting_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    ) -> Result<impl IntoResponse, RootErrors> {
    Ok (
        template_to_response(
            ArtPostingPage {
                user: None, //TODO: Connect with user system.
                original_uri
            }
        )
    )
}