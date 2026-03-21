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
        PostingSteps::UploadMetadata(mut given_page_lore) => {
            sanitize_recieved_lore_page(&mut given_page_lore);

            validate_recieved_lore_page(&given_page_lore).map_err(RootErrors::BadRequest)?;

            // Now that everything is valid, toss into database

            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            columns.push("belongs_to_category".into());
            values.push(&given_page_lore.parent_category_id);

            columns.push("slug".into());
            values.push(&given_page_lore.base.slug);

            columns.push("title".into());
            values.push(&given_page_lore.base.title);

            columns.push("description".into());
            values.push(&given_page_lore.base.description);

            columns.push("content".into());
            values.push(&given_page_lore.content);

            // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
            let query = format!(
                "INSERT INTO lore ({}) VALUES ({}) RETURNING id;",
                columns.join(","),
                (1..values.len() + 1)
                    .map(|i| format!("${i}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );

            let db_id: i32 = db_connection
                .query_one(&query, &values)
                .await
                .map_err(|err| {
                    eprintln!("[LORE CATEGORY UPLOAD] Executing SQL insert failed! {err:?}");
                    RootErrors::InternalServerError
                })?
                .get(0);

            println!(
                "[LORE PAGE UPLOAD] User {} (ID:{}) uploaded lore page {} (ID:{}, SLUG:{})",
                requesting_user.display_name,
                requesting_user.id,
                given_page_lore.base.title,
                db_id,
                given_page_lore.base.slug
            );

            Ok(Redirect::to(&format!("/lore/{}", given_page_lore.base.slug)).into_response())
        }
    }
}

fn sanitize_recieved_lore_page(given_lore_page: &mut PageLore) {
    given_lore_page.content = given_lore_page.content.trim().into();

    given_lore_page.base.description = given_lore_page
        .base
        .description
        .as_deref()
        .map(|description| description.trim().into());

    given_lore_page.base.title = given_lore_page.base.title.trim().into();
}

fn validate_recieved_lore_page(given_lore_page: &PageLore) -> Result<(), String> {
    if !crate::utils::is_valid_slug(&given_lore_page.base.slug) {
        return Err("Given an invalid page slug.".into());
    }

    // TODO

    // TODO: Validate the lore category exists

    Ok(())
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
            // TODO: Allow uploading stuff like category-specific images and thumbnails
            Err(RootErrors::BadRequest(
                "You can't upload files to lore sections, getthefuckouttahere.".into(),
            ))
        }
        PostingSteps::UploadMetadata(mut given_lore_category) => {
            sanitize_recieved_lore_category(&mut given_lore_category);

            validate_recieved_lore_category(&given_lore_category)
                .map_err(RootErrors::BadRequest)?;

            // Now that everything is valid, toss into database

            let mut columns: Vec<String> = Vec::new();
            let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

            columns.push("title".into());
            values.push(&given_lore_category.title);

            columns.push("description".into());
            values.push(&given_lore_category.description);

            columns.push("order_position".into());
            values.push(&given_lore_category.order_position);

            // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
            let query = format!(
                "INSERT INTO lore_category ({}) VALUES ({}) RETURNING id;",
                columns.join(","),
                (1..values.len() + 1)
                    .map(|i| format!("${i}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );

            let db_id: i32 = db_connection
                .query_one(&query, &values)
                .await
                .map_err(|err| {
                    eprintln!("[LORE CATEGORY UPLOAD] Executing SQL insert failed! {err:?}");
                    RootErrors::InternalServerError
                })?
                .get(0);

            println!(
                "[LORE CATEGORY UPLOAD] User {} (ID:{}) uploaded lore category {} (ID:{})",
                requesting_user.display_name, requesting_user.id, given_lore_category.title, db_id,
            );

            Ok(Redirect::to("/lore").into_response())
        }
    }
}

fn sanitize_recieved_lore_category(recieved_lore_category: &mut LoreCategory) {
    recieved_lore_category.title = recieved_lore_category.title.trim().into();

    recieved_lore_category.description = recieved_lore_category
        .description
        .as_ref()
        .map(|description| description.trim().into());
}

fn validate_recieved_lore_category(recieved_lore_category: &LoreCategory) -> Result<(), String> {
    // TODO

    Ok(())
}
