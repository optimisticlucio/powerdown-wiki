use utils::sql::PostState;
use crate::art::structs::{BaseArt, PageArt};
use crate::user::{User, UsermadePost};
use crate::utils::{self, template_to_response, PostingSteps};
use crate::{errs::RootErrors, ServerState};
use askama::Template;
use axum::extract::{OriginalUri, Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{http, Json};
use http::Uri;
use tokio::task::JoinSet;

const INSERT_INTO_ART_FILE_DB_QUERY: &str = "INSERT INTO art_file (belongs_to,internal_order,s3_key) VALUES ($1,$2,$3)";
const DELETE_FROM_ART_FILE_DB_QUERY: &str = "DELETE FROM art_file WHERE belongs_to=$1 AND internal_order=$2";

const ART_THUMBNAIL_COMPRESSION_SETTINGS: utils::file_compression::LossyCompressionSettings = utils::file_compression::LossyCompressionSettings {
                            max_width: Some(180),
                            max_height: Some(150),
                            quality: 60
                        };

/// Post Request Handler for art category.
#[axum::debug_handler]
pub async fn add_art(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_step): Json<PostingSteps<PageArt>>,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    // Who's trying to do this?
    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    // TODO: Disable uploading by non-logged in users after uploading the static site backlog.

    if requesting_user.as_ref().is_some_and(|user| !user.user_type.permissions().can_post_art) {
        return Err(RootErrors::Forbidden);
    }

    match posting_step {
        PostingSteps::RequestPresignedURLs { file_amount } => {
            give_user_presigned_s3_urls(file_amount, original_uri, cookie_jar, &state).await
        }
        PostingSteps::UploadMetadata(mut page_art) => {
            // Let's fix up some values that the user may have passed incorrectly.
            sanitize_recieved_page_art(&mut page_art, &state);

            // Now, let's make sure what we were given is even logical
            if let Err(err_explanation) = validate_recieved_page_art(&page_art) {
                return Err(RootErrors::BadRequest(
                    err_explanation,
                ));
            }

            // Check if this art already exists. If it does, throw an error.
            if BaseArt::get_by_slug(&db_connection, &page_art.base_art.slug).await.is_some() {
                return Err(RootErrors::BadRequest(
                    format!("The slug {} already exists.", &page_art.base_art.slug)
                ));
            }

            // Makes sense? Good. Our job now.
            // Let's build the query.
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            columns.push("post_state".into());
            values.push(&PostState::Processing);

            columns.push("page_slug".into());
            values.push(&page_art.base_art.slug);

            columns.push("creation_date".into());
            values.push(&page_art.creation_date);

            columns.push("title".into());
            values.push(&page_art.base_art.title);

            columns.push("creators".into());
            values.push(&page_art.base_art.creators);

            // We don't have the thumbnail quite yet, so just put in a garbage value
            columns.push("thumbnail".into());
            values.push(&"missing_thumbnail");

            
            if !page_art.tags.is_empty() {
                columns.push("tags".into());
                values.push(&page_art.tags);
            }

            columns.push("is_nsfw".into());
            values.push(&page_art.base_art.is_nsfw);

            let uploading_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

            if uploading_user.is_some() {
                columns.push("uploading_user_id".into());
                values.push(&uploading_user.as_ref().unwrap().id);
            }

            let sanitized_description = page_art.description.map(|description| {
                // TODO: SANITIZE FOR HTML/COMMONMARK INJECTION
                description.trim().to_string()
            }).filter(|description| !description.is_empty());

            if let Some(description) = &sanitized_description {
                // If we got here, the description is sanitized and not empty.
                columns.push("description".into());
                values.push(description);
            }

            // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
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
                    eprintln!("[ART UPLOAD] Initial DB upload failed! {:?}", err);
                    RootErrors::InternalServerError
                })?
                .get(0);

            // ---- We have the ID? process the thumbnail and update. ----
            let target_s3_folder = format!("art/{art_id}");

            let target_thumbnail_key = format!("{target_s3_folder}/thumbnail");

            let thumbnail_key = utils::move_and_lossily_compress_temp_s3_img(
                    &state.s3_client.clone(),
                    &state.config,
                    &page_art.base_art.thumbnail_key,
                    &state.config.s3_public_bucket,
                    &target_thumbnail_key,
                    Some(ART_THUMBNAIL_COMPRESSION_SETTINGS)
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Converting thumbnail of art {art_id} failed, {:?}",
                        err
                    );

                    // Delete the processing art before returning error.
                    let _ = db_connection.execute("DELETE FROM art WHERE id=$1", &[&art_id]);

                    RootErrors::InternalServerError
                })?;

            db_connection
                .execute(
                    "UPDATE art SET thumbnail=$1 WHERE id=$2",
                    &[&thumbnail_key, &art_id],
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Updating thumbnail key in DB of art {art_id} failed, {:?}",
                        err
                    );

                    // Delete the processing art before returning error.
                    let _ = db_connection.execute("DELETE FROM art WHERE id=$1", &[&art_id]);

                    RootErrors::InternalServerError
                })?;

            // ---- Now that the main art file is up, upload the individual art pieces. ----

            let mut art_upload_tasks = JoinSet::new();

            // The names of the files we're supposed to create, incase the upload fails.
            let temp_file_keys = page_art.art_keys;

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
                    .map_err(|_| RootErrors::InternalServerError)?;
                let target_s3_folder = target_s3_folder.clone();

                // tokio::spawn lets all the tasks run simultaneously, which is nice.
                art_upload_tasks.spawn(async move {
                    let file_key = format!(
                        "{target_s3_folder}/{}",
                        s3_key.split_terminator("/").last().unwrap()
                    );

                    let final_file_key = utils::move_temp_s3_file(
                        &s3_client,
                        &config,
                        &s3_key,
                        &public_bucket_key,
                        &file_key,
                    )
                    .await
                    .map_err(|err| format!("{:?}", err))?;

                    let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
                    values.push(&art_id);
                    values.push(&index);
                    values.push(&final_file_key);

                    db_connection
                        .execute(INSERT_INTO_ART_FILE_DB_QUERY, &values)
                        .await
                        .map_err(|err| format!("{:?}", err))?;

                    Ok(final_file_key)
                });
            }

            // Now collect everything that ran async, make sure nothing fucked up.
            let art_upload_results: Vec<Result<String, String>> = art_upload_tasks.join_all().await;

            // Let's get all the errors and the results
            let (final_art_keys, failed_upload_errs) = art_upload_results
                .into_iter()
                .fold((Vec::new(), Vec::new()), |(mut oks, mut errs), result| {
                    match result {
                        Ok(a) => oks.push(a),
                        Err(b) => errs.push(b),
                    }
                    (oks, errs)
                });

            if !failed_upload_errs.is_empty() {
                // TODO: Handle if part of this method fails. It's already a fail-method, so what then?

                let _ = utils::delete_keys_from_s3(
                    &state.s3_client.clone(),
                    &state.config.s3_public_bucket,
                    &final_art_keys).await;

                let _ = db_connection.execute("DELETE FROM art WHERE id=$1", &[&art_id]);

                eprintln!(
                    "[ART POST] Failed to move files from temp to permanent location! [{}]",
                    failed_upload_errs.join(", ")
                );

                return Err(RootErrors::InternalServerError);
            }

            // ---- Now that we finished, set the appropriate art state. ----

            db_connection
                .execute(
                    "UPDATE art SET post_state=$1 WHERE id=$2",
                    &[&PostState::Public, &art_id],
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Setting post state of id {art_id} to public failed?? {:?}",
                        err
                    );

                    // Delete the processing art before returning error.
                    let _ = db_connection.execute("DELETE FROM art WHERE id=$1", &[&art_id]);

                    RootErrors::InternalServerError
                })?;

            Ok(Redirect::to(&format!("/art/{}", page_art.base_art.slug)).into_response())
        }
    }
}

#[derive(Debug, Template)]
#[template(path = "art/new.html")]
pub struct ArtPostingPage {
    pub user: Option<User>,
    pub original_uri: Uri,

    /// Incase we're editing an existing page, pass the pageart here.
    pub art_being_modified: Option<PageArt>,

    /// The URL to which our upload button will be talking to. If empty, messages the current URI.
    pub target_button_url: Option<String>,
}

pub async fn art_posting_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    Ok(template_to_response(ArtPostingPage {
        user: User::easy_get_from_cookie_jar(&state, &cookie_jar).await?,
        original_uri,

        art_being_modified: None,
        target_button_url: None
    }))
}

pub async fn edit_art_put_request(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_step): Json<PostingSteps<PageArt>>,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    // Who's asking to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        None => return Err(RootErrors::Unauthorized),
        Some(requesting_user) => requesting_user,
    };

    let existing_art = match PageArt::get_by_slug(&db_connection, &art_slug).await {
        None => {
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                Some(requesting_user),
            ))
        }
        Some(existing_art) => existing_art,
    };

    // If they don't have permissions to do this, shoot back HTTP 403.
    if !(existing_art.can_be_modified_by(&requesting_user)) {
        return Err(RootErrors::Forbidden);
    }

    match posting_step {
        PostingSteps::RequestPresignedURLs { file_amount } => {
            give_user_presigned_s3_urls(file_amount, original_uri, cookie_jar, &state).await
        }
        PostingSteps::UploadMetadata(mut sent_page_art) => {
            // Let's fix up some values that the user may have passed incorrectly.
            sanitize_recieved_page_art(&mut sent_page_art, &state);

            // Now let's make sure what we were given is even logical
            if let Err(err_explanation) = validate_recieved_page_art(&sent_page_art) {
                return Err(RootErrors::BadRequest(
                    err_explanation,
                ));
            }

            // TODO: Check validity of art URLs. Don't move them yet, just ensure the user isn't fucking with us.

            // Now that everything is uploaded properly, let's start modifying what needs to be changed.
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            columns.push("post_state".into());
            values.push(&PostState::Processing);

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

            if sent_page_art.description != existing_art.description {
                columns.push("description".into());
                values.push(&sent_page_art.description);
            }

            // SAFETY: nothing user-written is passed into the string. User values are in `values`
            let query = format!(
                "UPDATE art SET {} WHERE id={};",
                columns.iter().enumerate()
                    .map(|(index, value)| format!("{}=${}", value, index+1))
                    .collect::<Vec<_>>()
                    .join(","),
                format!("${}", columns.len() + 1)
            );

            values.push(&existing_art.base_art.id);
            db_connection
                .execute(&query, &values)
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Updating metadata of art id {}, named \"{}\", failed. {:?} \nQUERY:{}\nPARAMS:{:?}",
                        &existing_art.base_art.id,
                        &existing_art.base_art.title,
                        err,
                        &query,
                        &values
                    );
                    RootErrors::InternalServerError
                })?;

            // Now let's reorder and reorganize the art. Go over all of the given art keys, and see which have been modified.
            let s3_client = state.s3_client.clone();
            let target_s3_folder = format!("art/{}", existing_art.base_art.id);

            for (art_key, new_art_key_index) in sent_page_art.art_keys.iter().zip(0i8..) {
                let previous_art_key_index = existing_art.art_keys
                    .iter()
                    .position(|old_key| old_key == art_key) // Get the index of where that art used to be.
                    ;

                // Converting "as i8" should be fine as long as no one puts over 127 art pieces in the same place.
                // As of writing this comment, I limit people to 35 at most, so we should be fine.
                if !previous_art_key_index.is_some_and(|previous_index| previous_index as i8 == new_art_key_index) {
                    // The new art at index i is unlike the previous art at index i.

                    let new_art_key = if previous_art_key_index.is_some() {
                        art_key.to_string()
                    } else {
                        // If it's new art, move it into place.
                        let target_file_key = format!(
                            "{target_s3_folder}/{}",
                            art_key.split_terminator("/").last().unwrap()
                        );

                        utils::move_temp_s3_file(
                            &s3_client,
                            &state.config,
                            art_key,
                            &state.config.s3_public_bucket,
                            &target_file_key
                        ).await
                        .map_err(|err| {
                            eprintln!("[MODIFY ART] Failed moving new art for \"{}\", id:{}. Err:{:?}", 
                                &existing_art.base_art.title,
                                &existing_art.base_art.id,
                                err);
                            
                            RootErrors::InternalServerError
                        })?
                    };

                    // Remove the DB entry for the current index.
                    db_connection
                        .execute(
                            DELETE_FROM_ART_FILE_DB_QUERY,
                            &[&existing_art.base_art.id, &((new_art_key_index+1) as i32)],
                        )
                        .await
                        .map_err(|err| {
                            eprintln!(
                                "[ART MODIFICATION] Deleting one of the records of art ID {} failed. {:?}",
                                existing_art.base_art.id,
                                err
                            );
                            RootErrors::InternalServerError
                        })?;

                    // Now insert a new value for this index.
                    db_connection
                        .execute(
                            INSERT_INTO_ART_FILE_DB_QUERY,
                            &[&existing_art.base_art.id, &((new_art_key_index+1) as i32), &new_art_key],
                        )
                        .await
                        .map_err(|err| {
                            eprintln!(
                                "[ART MODIFICATION] Adding a new record to art ID {} failed. {:?}",
                                existing_art.base_art.id,
                                err
                            );
                            RootErrors::InternalServerError
                        })?;
                }
            }
            
            // We modified all the existing records, cool. Now let's delete any records we didn't get to go over.
            if sent_page_art.art_keys.len() < existing_art.art_keys.len() {
                for leftover_index in sent_page_art.art_keys.len()..existing_art.art_keys.len() {   
                    db_connection
                        .execute(
                            DELETE_FROM_ART_FILE_DB_QUERY,
                            &[&existing_art.base_art.id, &(leftover_index as i32)],
                        )
                        .await
                        .map_err(|err| {
                            eprintln!(
                                "[ART MODIFICATION] Deleting one of the records of art ID {} failed. {:?}",
                                existing_art.base_art.id,
                                err
                            );
                            RootErrors::InternalServerError
                        })?;
                }
            }

            // Now that all the new art was moved in, let's delete the art that's no longer present.
            let art_keys_that_were_removed: Vec<String> = Vec::new(); // TODO: Get the removed art!

            // TODO: How do we handle this fail? If this fails the post is fine, it's just some leftovers on our side.
            utils::delete_keys_from_s3(
                &s3_client,
                &state.config.s3_public_bucket,
                &art_keys_that_were_removed)
                .await;

            // ---- Now that we finished, set the appropriate art state, and maybe update the thumbnail ----

            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
            
            columns.push("post_state".into());
            values.push(&PostState::Public);

            // I need to create new_thumbnail_key here so that, incase we use it, it can survive enough.
            let mut new_thumbnail_key = "".to_owned();
            if sent_page_art.base_art.thumbnail_key != existing_art.base_art.thumbnail_key {
                new_thumbnail_key = utils::move_and_lossily_compress_temp_s3_img(
                        &state.s3_client.clone(),
                        &state.config,
                        &sent_page_art.base_art.thumbnail_key,
                        &state.config.s3_public_bucket,
                        &existing_art.base_art.thumbnail_key,
                        Some(ART_THUMBNAIL_COMPRESSION_SETTINGS)
                    )
                    .await
                    .map_err(|err| {
                            eprintln!(
                                "[ART EDIT] Converting thumbnail of art {} failed, {:?}",
                                existing_art.base_art.id,
                                err
                            );
                            RootErrors::InternalServerError
                        }
                    )?;

                columns.push("thumbnail".into());
                values.push(&new_thumbnail_key);
            }

            values.push(&existing_art.base_art.id);

            let update_query = format!("UPDATE art SET {} WHERE id={};",
                columns.iter().enumerate()
                    .map(|(index, value)| format!("{}=${}", value, index+1))
                    .collect::<Vec<_>>()
                    .join(","),
                format!("${}", columns.len() + 1));

            db_connection
                .execute(
                    &update_query,
                    &values,
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[ART UPLOAD] Setting post state of id {} to public failed?? {:?}",
                        existing_art.base_art.id,
                        err
                    );
                    RootErrors::InternalServerError
                })?;

            Ok(Redirect::to(&format!("/art/{}", sent_page_art.base_art.slug)).into_response())
        }
    }
}

/// Given an amount of urls requested by the user, sends the user back the appropriate amount of new temp S3 presigned URLs. May also request an extra url for the thumbnail.
async fn give_user_presigned_s3_urls(
    requested_amount_of_urls: u8,
    original_uri: Uri,
    cookie_jar: tower_cookies::Cookies,
    state: &ServerState,
) -> Result<Response, RootErrors> {
    if requested_amount_of_urls > 35 {
        Err(RootErrors::BadRequest(
            "for the good of mankind, don't put that many art pieces in one post. split them up"
                .to_string(),
        ))
    } else {
        let presigned_urls = utils::get_temp_s3_presigned_urls(state, requested_amount_of_urls.into(), "art")
            .await.map_err(|err| {
                eprintln!("[ART POST STEP 1] {}", err);
                RootErrors::InternalServerError
            })?;

        // Send back the urls as a json.
        let response = serde_json::to_string(&utils::PresignedUrlsResponse {
            presigned_urls,
        })
        .unwrap();

        Ok(response.into_response())
    }
}

/// Given a user-created Page Art, validates that it makes sense. If it doesn't, returns a readable explanation why.
fn validate_recieved_page_art(recieved_page_art: &PageArt) -> Result<(), String> {
    if recieved_page_art.art_keys.is_empty() {
        return Err("Art page needs to have art in it".to_owned());
    }

    if recieved_page_art.base_art.creators.is_empty() {
        return Err("No artists given".to_owned());
    }

    if recieved_page_art.base_art.title.is_empty() {
        return Err("Title mustn't be empty.".to_owned());
    }

    const INVALID_ART_PAGE_TITLES: [&str; 2] = ["new", ""];
    if INVALID_ART_PAGE_TITLES.contains(&recieved_page_art.base_art.title.as_str()) {
        return Err("Invalid page title".to_owned());
    }

    if recieved_page_art.creation_date > chrono::offset::Local::now().date_naive() {
        return Err("Art can't be made in the future.".to_owned());
    }

    if recieved_page_art.base_art.thumbnail_key.is_empty() {
        return Err("Art must have thumbnail".to_owned());
    }

    Ok(())
}

/// Given a Page Art, cleans up any invalid or nonsensical values, such as empty strings for artist names.
/// NOTE: Does not make sure the values make _logical_ sense, only that we don't deal with trivially incorrect data.
fn sanitize_recieved_page_art(recieved_page_art: &mut PageArt, state: &ServerState) {
    // Clean up any empty tags
    recieved_page_art.tags = recieved_page_art.tags.iter().filter_map(|tag| {
        // SAFETY: The code doesn't pass the tags directly anywhere and are filtered by askama,
        // as they never have any parsing-relevant info in them. Well, _shouldn't_ have.
        // Therefore we don't need to sanitize them here.
        let trimmed_tag = tag.trim();
        
        if trimmed_tag.is_empty() {
            None
        }
        else {
            Some(trimmed_tag.to_lowercase())
        }
    }).collect();

    // Clean up any empty artist names.
    recieved_page_art.base_art.creators = recieved_page_art.base_art.creators.iter().filter_map(|creator_name| {
        // SAFETY: artist names are never passed with the "| safe" tag to askama, assumed to be dangerous anyways.
        let trimmed_name = creator_name.trim();
        
        if trimmed_name.is_empty() {
            None
        }
        else {
            Some(trimmed_name.to_string())
        }
    }).collect();

    // Get only the keys from the URLs the user gave us.
    // We don't need to raise an error if the host is wrong bc if the host is wrong, the key _has_ got to be wrong too.
    // If the host is wrong but the key is correct I legitimately have no idea what the fuck the user is doing.

    recieved_page_art.art_keys = recieved_page_art.art_keys
        .iter()
        .filter_map(|url| utils::clean_passed_key(url, state))
        .collect();

    // If this is invalid, it returns an empty string. I know, not great, is handled by the verification function.
    recieved_page_art.base_art.thumbnail_key = utils::clean_passed_key(&recieved_page_art.base_art.thumbnail_key, state).unwrap_or_default();
}
