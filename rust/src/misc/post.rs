use crate::misc::structs::MiscItem;
use crate::{RootErrors, ServerState, User};
use crate::utils::{PostingSteps};
use axum::extract::{State};
use axum::response::{IntoResponse, Response};
use axum::Json;

/// Post Request Handler for editing lore categories.
#[axum::debug_handler]
pub async fn edit_misc_section(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(posting_steps): Json<PostingSteps<Vec<MiscItem>>>,
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

            todo!()
        },
        PostingSteps::UploadMetadata(mut misc_items) => {
            let existing_items = MiscItem::get_all(&db_connection).await;

            // Let's build one big transaction so we don't have a bunch of inbetween moments.
            let sql_transaction = db_connection.transaction().await.map_err(|err| {
                eprintln!("[EDIT MISC SECTION] Errored trying to create an SQL Transaction! {err:?}");
                RootErrors::InternalServerError
            })?;

            // Firstly let's sanitize and validate everything the user passed us.

            for misc_item in &mut misc_items {
                sanitize_recieved_misc_item(misc_item);

                validate_recieved_misc_item(misc_item).map_err(RootErrors::BadRequest)?;
            }

            // TODO: Validate the given order positions make sense. No duplicates and the like.

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

                // TODO: HANDLE NEW THUMBNAIL

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

                // TODO: HANDLE NEW THUMBNAIL

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

            Ok(http::StatusCode::CREATED.into_response())
        }
    }

    
}

fn sanitize_recieved_misc_item(recieved_misc_item: &mut MiscItem) {
    todo!()
}

fn validate_recieved_misc_item(recieved_misc_item: &MiscItem) -> Result<(), String> {
    if recieved_misc_item.title.len() > 26 {
        return Err("The title should be at most 26 characters long.".to_string());
    }
    
    todo!()
}
