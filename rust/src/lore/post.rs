use super::structs::PageLore;
use crate::lore::structs::LoreCategory;
use crate::{utils::PostingSteps, RootErrors, ServerState, User};
use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;

/// Post Request Handler for adding new lore pages.
#[axum::debug_handler]
pub async fn add_lore_page(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_step): Json<PostingSteps<PageLore>>,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    // Who's trying to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        Some(user) => user,
        None => return Err(RootErrors::Unauthorized),
    };

    if !requesting_user.user_type.permissions().can_modify_lore {
        return Err(RootErrors::Forbidden);
    }

    match posting_step {
        PostingSteps::RequestPresignedURLs { file_amount: _ } => {
            // TODO: Allow uploading stuff like post-specific images and thumbnails
            Err(RootErrors::BadRequest(
                "You can't upload files to lore sections, getthefuckouttahere.".into(),
            ))
        }
        PostingSteps::UploadMetadata(given_page_lore) => {
            // TODO: Validate the lore category exists

            // TODO: Validate the fields make sense.

            // TODO: Shove that shit in the database

            println!(
                "[LORE PAGE UPLOAD] User {} (ID:{}) uploaded lore page {} (ID:{}, SLUG:{})",
                requesting_user.display_name,
                requesting_user.id,
                given_page_lore.base.title,
                given_page_lore.base.id, // TODO: Set to correct ID
                given_page_lore.base.slug
            );

            Ok(Redirect::to(&format!("/lore/{}", given_page_lore.base.slug)).into_response())
        }
    }
}

/// Post Request Handler for adding new lore categories.
#[axum::debug_handler]
pub async fn add_lore_category(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_step): Json<PostingSteps<LoreCategory>>,
) -> Result<Response, RootErrors> {
    let db_connection = state
        .db_pool
        .get()
        .await
        .map_err(|_| RootErrors::InternalServerError)?;

    // Who's trying to do this?
    let requesting_user = match User::get_from_cookie_jar(&db_connection, &cookie_jar).await {
        Some(user) => user,
        None => return Err(RootErrors::Unauthorized),
    };

    if !requesting_user.user_type.permissions().can_modify_lore {
        return Err(RootErrors::Forbidden);
    }

    match posting_step {
        PostingSteps::RequestPresignedURLs { file_amount: _ } => {
            // TODO: Allow uploading stuff like post-specific images and thumbnails
            Err(RootErrors::BadRequest(
                "You can't upload files to lore sections, getthefuckouttahere.".into(),
            ))
        }
        PostingSteps::UploadMetadata(given_lore_category) => {
            // TODO: Validate the lore category exists

            // TODO: Validate the fields make sense.

            // TODO: Shove that shit in the database

            println!(
                "[LORE CATEGORY UPLOAD] User {} (ID:{}) uploaded lore category {} (ID:{})",
                requesting_user.display_name,
                requesting_user.id,
                given_lore_category.title,
                given_lore_category.id, // TODO: Set to correct ID
            );

            Ok(Redirect::to("/lore").into_response())
        }
    }
}
