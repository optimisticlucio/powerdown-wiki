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

            if PageLore::get_from_slug(&db_connection, &given_page_lore.base.slug)
                .await
                .is_some()
            {
                return Err(RootErrors::BadRequest(format!(
                    "A lore page with the slug {} already exists.",
                    given_page_lore.base.slug
                )));
            }

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

/// Post Request Handler for editing lore categories.
#[axum::debug_handler]
pub async fn edit_lore_categories(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(mut lore_categories): Json<Vec<LoreCategory>>,
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

    if !requesting_user.user_type.permissions().can_modify_lore {
        return Err(RootErrors::Forbidden);
    }

    let existing_categories = LoreCategory::get_all_categories(&db_connection).await;

    // Let's build one big transaction so we don't have a bunch of inbetween moments.
    let sql_transaction = db_connection.transaction().await.map_err(|err| {
        eprintln!("[EDIT LORE CATEGORIES] Errored trying to create an SQL Transaction! {err:?}");
        RootErrors::InternalServerError
    })?;

    // Firstly let's sanitize and validate everything the user passed us.

    for category in &mut lore_categories {
        sanitize_recieved_lore_category(category);

        validate_recieved_lore_category(category).map_err(RootErrors::BadRequest)?;
    }

    // TODO: Validate the given order positions make sense. No duplicates and the like.

    // Let's see if any of these are new, and add them to the db. As only positive id values are DB Generated, look for any nonpositive ones.
    let (new_categories, modified_categories): (Vec<_>, Vec<_>) = lore_categories
        .into_iter()
        .partition(|category| category.id <= 0);

    for modified_category in modified_categories {
        // Check if anything changed. If so, toss a transaction on the pile.

        let mut columns: Vec<String> = Vec::new();
        let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

        let Some(existing_category) = existing_categories
            .iter()
            .find(|category| category.id == modified_category.id)
        else {
            return Err(RootErrors::BadRequest(format!(
                "Sent category named \"{}\" has nonexistent ID - {}",
                modified_category.title, modified_category.id
            )));
        };

        if existing_category.title != modified_category.title {
            columns.push("title".into());
            values.push(&modified_category.title);
        }

        if existing_category.description != modified_category.description {
            columns.push("description".into());
            values.push(&modified_category.description);
        }

        if existing_category.order_position != modified_category.order_position {
            columns.push("order_position".into());
            values.push(&modified_category.order_position);
        }

        if columns.is_empty() {
            continue;
        }

        values.push(&modified_category.id);

        // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
        let query = format!(
            "UPDATE lore_category SET {} WHERE id=${};",
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
                requesting_user.display_name, requesting_user.id, existing_category.title,
            );
    }

    for new_category in new_categories {
        let mut columns: Vec<String> = Vec::new();
        let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

        columns.push("title".into());
        values.push(&new_category.title);

        columns.push("description".into());
        values.push(&new_category.description);

        columns.push("order_position".into());
        values.push(&new_category.order_position);

        // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
        let query = format!(
            "INSERT INTO lore_category ({}) VALUES ({}) RETURNING id;",
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
                eprintln!("[EDIT LORE CATEGORIES] Adding SQL insert query failed! {err:?}");
                RootErrors::InternalServerError
            })?;

        println!(
                "[EDIT LORE CATEGORIES] User {} (ID:{}) is attempting to upload lore category {}. Proceeding with transaction.",
                requesting_user.display_name, requesting_user.id, new_category.title,
            );
    }

    // Now run it all and pray for the best.
    sql_transaction.commit().await.map_err(|err| {
        eprintln!("[EDIT LORE CATEGORIES] Errored trying to run SQL Transaction! {err:?}");
        RootErrors::InternalServerError
    })?;

    println!(
        "[EDIT LORE CATEGORIES User {} (ID:{}) successfully modified lore categories.",
        requesting_user.display_name, requesting_user.id
    );

    Ok(http::StatusCode::CREATED.into_response())
}

fn sanitize_recieved_lore_category(recieved_lore_category: &mut LoreCategory) {
    recieved_lore_category.title = recieved_lore_category.title.trim().into();

    recieved_lore_category.description = recieved_lore_category
        .description
        .as_ref()
        .map(|description| description.trim().into());
}

fn validate_recieved_lore_category(recieved_lore_category: &LoreCategory) -> Result<(), String> {
    if recieved_lore_category.title.len() > 20 {
        return Err("The title should be at most 20 characters long.".to_string());
    }

    if recieved_lore_category
        .description
        .as_ref()
        .is_some_and(|desciption| desciption.len() > 256)
    {
        return Err("Don't write the bible in the description! 256 character max length.".into());
    }

    // TODO

    Ok(())
}
