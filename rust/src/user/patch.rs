//! # Modifying Users
//!
//! This file is for PATCH requests for a given user, to modify their values like their permission level, pfp, username, etc.

use super::structs::UserType;
use crate::utils::file_compression::LossyCompressionSettings;
use crate::utils::{
    get_temp_s3_presigned_urls, template_to_response, MoveTempS3FileErrs, PostingSteps,
    PresignedUrlsResponse,
};
use crate::{utils, RootErrors, ServerState, User};
use askama::Template;
use axum::extract::{OriginalUri, Path, State};
use axum::response::{IntoResponse, Response};
use axum::{http, Json};
use http::Uri;
use serde::Deserialize;

const PROFILE_PICTURE_COMPRESSION_SETTINGS: LossyCompressionSettings = LossyCompressionSettings {
    max_height: Some(150),
    max_width: Some(150),
    quality: 85,
};

#[axum::debug_handler]
pub async fn patch_user(
    Path(user_id): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_step): Json<PostingSteps<ModifiableUserInfo>>,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    // Who's trying to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        Some(requesting_user) => requesting_user,
        None => {
            return Err(RootErrors::Unauthorized);
        }
    };

    // Who are they trying to modify?
    let user_id = match user_id.parse() {
        Err(_) => {
            // If the parse failed, it's 100% a nonexistent user ID. Shoot back 404.
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                Some(requesting_user),
            ));
        }
        Ok(id) => id,
    };
    let modified_user = match User::get_by_id(&db_connection, &user_id).await {
        Some(user) => user,
        None => {
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                Some(requesting_user),
            ));
        }
    };

    if !modified_user.can_have_visible_data_modified_by(&requesting_user) {
        return Err(RootErrors::Forbidden);
    }

    // Ok we know who's doing it, to who, and that they have *some* permissions to modify this user atleast. Let's start flow.

    match posting_step {
        PostingSteps::RequestPresignedURLs { file_amount } => {
            let presigned_urls = get_temp_s3_presigned_urls(&state, file_amount as u32, "user")
                .await
                .map_err(|err| {
                    eprintln!(
                        "[MODIFY USER INFO] Failed getting {} s3 urls! {}",
                        file_amount, err
                    );
                    RootErrors::InternalServerError
                })?;

            Ok(
                serde_json::to_string(&PresignedUrlsResponse { presigned_urls })
                    .unwrap()
                    .into_response(),
            )
        }
        PostingSteps::UploadMetadata(modified_user_info) => {
            // Let's build an update query.
            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            if let Some(passed_user_type) = &modified_user_info.user_type {
                // Can't modify your own type.
                if modified_user == requesting_user {
                    return Err(RootErrors::Forbidden);
                }

                // No one can promote to superadmin, that's only done through direct db modification.
                if passed_user_type == &UserType::Superadmin {
                    return Err(RootErrors::Forbidden);
                }

                // Check if it's an admin promotion.
                if passed_user_type == &UserType::Admin
                    && !requesting_user.user_type.permissions().can_promote_to_admin
                {
                    return Err(RootErrors::Forbidden);
                }

                // If we got here, valid modification. Let's go.
                columns.push("user_type".to_string());
                values.push(passed_user_type);
            };

            // Creating the variable outside the if statement for lifetime reasons.
            let new_pfp_key: String;
            if let Some(passed_pfp_key) = &modified_user_info.pfp_temp_key {
                let target_file_key = format!("user/{}/profile_picture", &modified_user.id);
                let cleaned_pfp_key = match utils::clean_passed_key(passed_pfp_key, &state) {
                    Some(clean_key) => clean_key,
                    None => {
                        return Err(RootErrors::BadRequest(
                            "Passed invalid pfp key.".to_string(),
                        ));
                    }
                };

                new_pfp_key = utils::move_and_lossily_compress_temp_s3_img(
                    &state.s3_client,
                    &state.config,
                    &cleaned_pfp_key,
                    &state.config.s3_public_bucket,
                    &target_file_key,
                    Some(PROFILE_PICTURE_COMPRESSION_SETTINGS),
                )
                .await
                .map_err(|err| match err {
                    MoveTempS3FileErrs::UnknownFiletype => {
                        RootErrors::BadRequest("Invalid file for profile picture.".to_string())
                    }
                    _ => {
                        eprintln!("[UPDATE USER INFO] Failed to compress user PFP! {}", err);
                        RootErrors::InternalServerError
                    }
                })?;

                columns.push("profile_picture_s3_key".to_string());
                values.push(&new_pfp_key);
            };

            let sanitized_display_name: String;
            if let Some(passed_display_name) = &modified_user_info.display_name {
                sanitized_display_name = match sanitize_display_name(passed_display_name) {
                    None => {
                        return Err(RootErrors::BadRequest(
                            "Invalid username passed.".to_string(),
                        ))
                    }
                    Some(name) => name,
                };

                columns.push("display_name".to_string());
                values.push(&sanitized_display_name);
            }

            let sanitized_creator_name: String;
            if let Some(creator_name) = &modified_user_info.creator_name {
                // You can't modify your own creator name. Too easy to fuck shit up like that.
                if modified_user == requesting_user {
                    return Err(RootErrors::Forbidden);
                }

                columns.push("creator_name".to_string());
                // TODO: I should probably make this into its own function later rather than
                // just reusing the display name function.
                if let Some(creator_name) = sanitize_display_name(creator_name) {
                    sanitized_creator_name = creator_name;
                    values.push(&sanitized_creator_name);
                } else {
                    // If " " is passed (or an invalid name), set to no creator name.
                    values.push(&"NULL");
                }
            }

            // Did we actually do anything?
            if columns.len() == 0 {
                return Err(RootErrors::BadRequest(
                    "No modifiable data passed.".to_string(),
                ));
            }

            // Send the query and pray
            let update_query = format!(
                "UPDATE site_user SET {} WHERE id={};",
                columns
                    .iter()
                    .enumerate()
                    .map(|(index, value)| format!("{}=${}", value, index + 1))
                    .collect::<Vec<_>>()
                    .join(","),
                format!("${}", columns.len() + 1)
            );

            values.push(&modified_user.id);

            db_connection
                .execute(
                    &update_query,
                    &values,
                )
                .await
                .map_err(|err| {
                    eprintln!(
                        "[USER MODIFICATION] Changing properties of userid {} by userid {} failed when pushing to DB: {:?}",
                        &modified_user.id,
                        &requesting_user.id,
                        err
                    );
                    RootErrors::InternalServerError
                })?;

            Ok(axum::http::StatusCode::NO_CONTENT.into_response())
        }
    }
}

/// The info I expect to recieve from the end-user about what should be modified for this given user.
/// Making this one a separate struct bc I don't want to give any user the full info about another user,
/// unlike Art and Characters where they just send back the whole thing and I see what needs to be modified.
#[derive(Deserialize, Debug)]
pub struct ModifiableUserInfo {
    /// New display name
    #[serde(default)]
    display_name: Option<String>,
    /// S3 key, in the temp area, pointing to a new and spankin' pfp for this user.
    #[serde(default)]
    pfp_temp_key: Option<String>,
    /// The new user type for this user. IF THIS IS PASSED, ASSURE IT'S AN ADMIN DOING THE ACTION!!
    #[serde(default)]
    user_type: Option<UserType>,
    /// The username that refers to this user when it comes to characters or art posts and such.
    /// If an empty string is passed, set to NULL in the DB.
    #[serde(default)]
    creator_name: Option<String>,
}

// Given a display name by the user, cleans it up. Returns None if the username is invalid or can't be easily cleaned.
fn sanitize_display_name(display_name: &str) -> Option<String> {
    const ALLOWED_SPECIAL_CHARACTERS: &[char] = &[' ', '_', '-', '.', '!', '?', '(', ')', ':'];

    // Right now, I just refuse to handle anything non-ascii. I can fix it later.
    if !display_name.is_ascii() {
        return None;
    }

    // Collapse any multiple-spaces in a row.
    let collapsed_display_name = display_name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Ensure it's not too long
    if collapsed_display_name.len() > 36 {
        return None;
    }

    // Ensure it's not too short
    if collapsed_display_name.is_empty() {
        return None;
    }

    // Ensure it's mostly valid characters
    for c in collapsed_display_name.chars() {
        if !c.is_ascii_alphanumeric() && !ALLOWED_SPECIAL_CHARACTERS.contains(&c) {
            return None;
        }
    }

    Some(collapsed_display_name.to_string())
}

#[axum::debug_handler]
pub async fn modify_user_page(
    Path(user_id): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    // Who's trying to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        Some(requesting_user) => requesting_user,
        None => {
            return Err(RootErrors::Unauthorized);
        }
    };

    // Who are they trying to modify?
    let user_id = match user_id.parse() {
        Err(_) => {
            // If the parse failed, it's 100% a nonexistent user ID. Shoot back 404.
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                Some(requesting_user),
            ));
        }
        Ok(id) => id,
    };

    let modified_user = match User::get_by_id(&db_connection, &user_id).await {
        Some(user) => user,
        None => {
            return Err(RootErrors::NotFound(
                original_uri,
                cookie_jar,
                Some(requesting_user),
            ));
        }
    };

    if !modified_user.can_have_visible_data_modified_by(&requesting_user) {
        return Err(RootErrors::Forbidden);
    }

    Ok(template_to_response(ModifyUserPage {
        user: Some(requesting_user.clone()),
        original_uri,

        modifying_user: requesting_user,

        viewed_user: modified_user,
    }))
}

#[derive(Debug, Template)]
#[template(path = "user/modify.html")]
struct ModifyUserPage {
    user: Option<User>,
    original_uri: Uri,

    viewed_user: User,
    modifying_user: User,
}
