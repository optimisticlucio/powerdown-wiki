use crate::misc::structs::MiscItem;
use crate::utils::{self, get_temp_s3_presigned_urls, PostingSteps, PresignedUrlsResponse};
use crate::{RootErrors, ServerState, User};
use ammonia::Url;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RecievedMiscItems {
    misc_items: Vec<MiscItem>,
}

const THUMBNAIL_COMPRESSION_SETTINGS: utils::file_compression::LossyCompressionSettings =
    utils::file_compression::LossyCompressionSettings {
        max_width: Some(250),
        max_height: Some(250),
        quality: 85,
    };

/// Post Request Handler for editing lore categories.
#[axum::debug_handler]
pub async fn edit_misc_section(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_steps): Json<PostingSteps<RecievedMiscItems>>,
) -> Result<Response, RootErrors> {
    let mut db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    // Who's trying to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        Some(user) => user,
        None => return Err(RootErrors::Unauthorized),
    };

    if !requesting_user.user_type.permissions().can_modify_misc {
        return Err(RootErrors::Forbidden);
    }

    match posting_steps {
        PostingSteps::RequestPresignedURLs { file_amount } => {
            let presigned_urls = get_temp_s3_presigned_urls(&state, file_amount.into(), "misc")
                .await
                .map_err(|err| {
                    eprintln!("[EDIT MISC SECTION] Failed to get presigned URLs! {err}");
                    RootErrors::InternalServerError
                })?;

            // Now return the presigned urls as a json
            Ok(
                serde_json::to_string(&PresignedUrlsResponse { presigned_urls })
                    .unwrap()
                    .into_response(),
            )
        }
        PostingSteps::UploadMetadata(RecievedMiscItems { mut misc_items }) => {
            let existing_items = MiscItem::get_all(&db_connection).await;

            // Let's build one big transaction so we don't have a bunch of inbetween moments.
            let sql_transaction = db_connection.transaction().await.map_err(|err| {
                eprintln!(
                    "[EDIT MISC SECTION] Errored trying to create an SQL Transaction! {err:?}"
                );
                RootErrors::InternalServerError
            })?;

            // Firstly let's sanitize and validate everything the user passed us.

            for misc_item in &mut misc_items {
                sanitize_recieved_misc_item(misc_item, &state);

                validate_recieved_misc_item(misc_item).map_err(RootErrors::BadRequest)?;
            }

            // TODO: Validate the given order positions make sense. No duplicates and the like.

            let mut s3_keys_to_delete: Vec<String> = Vec::new();
            let s3_client = state.s3_client.clone();
            let target_s3_folder = "misc/thumbnails";

            // Let's see if any of these are new, and add them to the db. As only positive id values are DB Generated, look for any nonpositive ones.
            let (new_items, modified_items): (Vec<_>, Vec<_>) = misc_items
                .into_iter()
                .partition(|misc_item| misc_item.id <= 0);

            for modified_item in modified_items {
                // Check if anything changed. If so, toss a transaction on the pile.

                let mut columns: Vec<String> = Vec::new();
                let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

                let Some(existing_item) = existing_items
                    .iter()
                    .find(|misc_item| misc_item.id == modified_item.id)
                else {
                    return Err(RootErrors::BadRequest(format!(
                        "Sent item named \"{}\" has nonexistent ID - {}",
                        modified_item.title, modified_item.id
                    )));
                };

                if existing_item.title != modified_item.title {
                    columns.push("title".into());
                    values.push(&modified_item.title);
                }

                if existing_item.description != modified_item.description {
                    columns.push("description".into());
                    values.push(&modified_item.description);
                }

                if existing_item.order_position != modified_item.order_position {
                    columns.push("order_position".into());
                    values.push(&modified_item.order_position);
                }

                if existing_item.url != modified_item.url {
                    columns.push("url".into());
                    values.push(&modified_item.url);
                }

                // Incase it needs to be used.
                let thumbnail_s3_key: String;
                if existing_item.thumbnail_url != modified_item.thumbnail_url {
                    columns.push("thumbnail".into());
                    if let Some(thumbnail_url) = &modified_item.thumbnail_url {
                        // Upload the thumbnail to the DB assuming everything else will work out. We're optimistic here. Also, lazy.

                        // Move thumbnail.
                        let random_string = crate::utils::get_random_string(16);
                        let thumbnail_target_s3_key = format!("{target_s3_folder}/{random_string}");
                        thumbnail_s3_key = match utils::move_and_lossily_compress_temp_s3_img(
                            &s3_client,
                            &state.config,
                            thumbnail_url,
                            &state.config.s3_public_bucket,
                            &thumbnail_target_s3_key,
                            Some(THUMBNAIL_COMPRESSION_SETTINGS),
                        )
                        .await
                        {
                            Ok(x) => x,
                            Err(err) => {
                                let item_id = existing_item.id;

                                eprintln!(
                                "[EDIT LORE CATEGORIES] Converting thumbnail of existing misc item (ID: {item_id}) failed, temp file URL is {thumbnail_url}, {err:?}",
                                );

                                return Err(RootErrors::InternalServerError);
                            }
                        };

                        values.push(&thumbnail_s3_key);
                    } else {
                        values.push(&None::<String>);
                    }

                    // Whatever we do here, we need to get rid of the prev image.
                    if let Some(previous_image_key) = &existing_item.thumbnail_url {
                        s3_keys_to_delete.push(previous_image_key.clone());
                    }
                }

                if columns.is_empty() {
                    continue;
                }

                values.push(&modified_item.id);

                // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
                let query = format!(
                    "UPDATE misc SET {} WHERE id=${};",
                    columns
                        .iter()
                        .enumerate()
                        .map(|(index, value)| format!("{}=${}", value, index + 1))
                        .collect::<Vec<_>>()
                        .join(","),
                    columns.len() + 1
                );

                sql_transaction
                    .execute(&query, &values)
                    .await
                    .map_err(|err| {
                        eprintln!("[EDIT LORE CATEGORIES] Adding SQL update query failed! {err:?}");
                        RootErrors::InternalServerError
                    })?;

                println!(
                        "[EDIT LORE CATEGORIES] User {} (ID:{}) is attempting to edit lore category {}. Proceeding with transaction.",
                        requesting_user.display_name, requesting_user.id, existing_item.title,
                    );
            }

            for new_item in new_items {
                let mut columns: Vec<String> = Vec::new();
                let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

                columns.push("title".into());
                values.push(&new_item.title);

                columns.push("description".into());
                values.push(&new_item.description);

                columns.push("order_position".into());
                values.push(&new_item.order_position);

                columns.push("url".into());
                values.push(&new_item.url);

                // Incase it needs to be used.
                let thumbnail_s3_key: String;
                if let Some(thumbnail_url) = &new_item.thumbnail_url {
                    // Upload the thumbnail to the DB assuming everything else will work out. We're optimistic here. Also, lazy.

                    // Move thumbnail.
                    let random_string = crate::utils::get_random_string(16);
                    let thumbnail_target_s3_key = format!("{target_s3_folder}/{random_string}");
                    thumbnail_s3_key = match utils::move_and_lossily_compress_temp_s3_img(
                        &s3_client,
                        &state.config,
                        thumbnail_url,
                        &state.config.s3_public_bucket,
                        &thumbnail_target_s3_key,
                        Some(THUMBNAIL_COMPRESSION_SETTINGS),
                    )
                    .await
                    {
                        Ok(x) => x,
                        Err(err) => {
                            eprintln!(
                            "[EDIT LORE CATEGORIES] Converting thumbnail of new misc item failed, temp file URL is {thumbnail_url}, {err:?}",
                            );

                            return Err(RootErrors::InternalServerError);
                        }
                    };

                    columns.push("thumbnail".into());
                    values.push(&thumbnail_s3_key);
                }

                // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
                let query = format!(
                    "INSERT INTO misc ({}) VALUES ({});",
                    columns.join(","),
                    (1..values.len() + 1)
                        .map(|i| format!("${i}"))
                        .collect::<Vec<_>>()
                        .join(",")
                );

                sql_transaction
                    .execute(&query, &values)
                    .await
                    .map_err(|err| {
                        eprintln!("[EDIT MISC ITEMS] Adding SQL insert query failed! {err:?}");
                        RootErrors::InternalServerError
                    })?;

                println!(
                        "[EDIT MISC ITEMS] User {} (ID:{}) is attempting to upload item {}. Proceeding with transaction.",
                        requesting_user.display_name, requesting_user.id, new_item.title,
                    );
            }

            // Now run it all and pray for the best.
            sql_transaction.commit().await.map_err(|err| {
                eprintln!("[EDIT MISC ITEMS] Errored trying to run SQL Transaction! {err:?}");
                RootErrors::InternalServerError
            })?;

            println!(
                "[EDIT MISC ITEMS] User {} (ID:{}) successfully modified misc section.",
                requesting_user.display_name, requesting_user.id
            );

            // If this fails - too bad.
            let _ = utils::delete_keys_from_s3(
                &state.s3_client.clone(),
                &state.config.s3_public_bucket,
                &s3_keys_to_delete,
            )
            .await.map_err(|err| {
                eprintln!("[EDIT MISC ITEMS] Failed to delete old thumbnails. Continuing anyways. ERR: {err:?}");
                RootErrors::InternalServerError
            });

            Ok(http::StatusCode::CREATED.into_response())
        }
    }
}

fn sanitize_recieved_misc_item(recieved_misc_item: &mut MiscItem, state: &ServerState) {
    recieved_misc_item.description = recieved_misc_item.description.trim().to_string();

    recieved_misc_item.title = recieved_misc_item.title.trim().to_string();

    recieved_misc_item.url = recieved_misc_item.url.trim().to_lowercase().to_string();

    recieved_misc_item.thumbnail_url = recieved_misc_item
        .thumbnail_url
        .as_deref()
        .and_then(|thumbnail_url| crate::utils::clean_passed_key(thumbnail_url, state));
}

fn validate_recieved_misc_item(recieved_misc_item: &MiscItem) -> Result<(), String> {
    if recieved_misc_item.title.len() > 26 {
        return Err("The title should be at most 26 characters long.".to_string());
    }

    if !is_valid_misc_item_path(&recieved_misc_item.url) {
        return Err("The target URL is invalid. It should either be a full path (\"https://www.whatever.com\"), or a relative path if it's a page in this site (\"/target/location\").".to_string());
    }

    if recieved_misc_item.description.len() > 256 {
        return Err(
            "Dont write the fuckin bible in there; description should be at most 256 characters."
                .to_string(),
        );
    }

    // TODO: Validate thumbnail makes sense

    Ok(())
}

/// Checks whether the given misc item's pointed URL is one we accept.
/// Right now checks if it's a proper URL, or a relative path starting with /.
fn is_valid_misc_item_path(misc_item_url: &str) -> bool {
    // Check if it's a complete path
    if Url::parse(misc_item_url).is_ok_and(|url| matches!(url.scheme(), "http" | "https")) {
        return true;
    }

    // Check if it's a relative path
    misc_item_url.starts_with("/")
        && Url::parse("https://www.google.com")
            .unwrap()
            .join("misc_item_url")
            .is_ok()
}
