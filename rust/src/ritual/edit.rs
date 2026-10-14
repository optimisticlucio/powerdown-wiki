use super::RitualCharacterData;
use crate::{
    characters::BaseCharacter, utils::template_to_response, RootErrors, ServerState, User,
};
use askama::Template;
use axum::{
    extract::{Json, OriginalUri, State},
    response::{IntoResponse, Response},
};
use http::Uri;
use serde::Deserialize;

#[axum::debug_handler]
pub async fn edit_ritual_data_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    let db_connection = state.db_pool.get().await.unwrap();

    let requesting_user = User::get_from_cookie_jar(&db_connection, &cookie_jar).await;

    let Some(requesting_user) = requesting_user else {
        return Err(RootErrors::Unauthorized);
    };

    if !requesting_user.user_type.permissions().can_update_ritual {
        return Err(RootErrors::Forbidden);
    };

    let all_characters = BaseCharacter::get_active_characters(&db_connection)
        .await
        .map_err(|err| {
            eprintln!("[RITUAL EDIT PAGE] Failed to read character data! {err:?}");
            RootErrors::InternalServerError
        })?;

    let ritualist_data = RitualCharacterData::get_all_ritualists(&db_connection)
        .await
        .map_err(|err| {
            eprintln!("[RITUAL EDIT PAGE] Failed to read ritualist data! {err:?}");
            RootErrors::InternalServerError
        })?;

    let base_characters_in_ritual: Vec<&BaseCharacter> = ritualist_data
        .iter()
        .map(|ritual_data| &ritual_data.page_character.base_character)
        .collect();

    let base_characters_without_ritual_data: Vec<BaseCharacter> = all_characters
        .into_iter()
        .filter(|character| !base_characters_in_ritual.contains(&character))
        .collect();

    Ok(template_to_response(RitualEdit {
        user: Some(requesting_user),
        original_uri,

        characters_with_ritual_data: ritualist_data,

        characters_without_ritual_data: base_characters_without_ritual_data,
    }))
}

#[derive(Debug, Template)]
#[template(path = "misc/ritual/edit.html")]
struct RitualEdit {
    user: Option<User>,
    original_uri: Uri,

    characters_with_ritual_data: Vec<RitualCharacterData>,

    characters_without_ritual_data: Vec<BaseCharacter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RitualistEditData {
    character_id: i32,
    power_name: String,
    power_description: String,
}

/// Post Request Handler for editing the ritual data.
#[axum::debug_handler]
pub async fn edit_ritual_data(
    State(state): State<ServerState>,
    cookie_jar: tower_cookies::Cookies,
    Json(ritualist_data): Json<Vec<RitualistEditData>>,
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

    if !requesting_user.user_type.permissions().can_update_ritual {
        return Err(RootErrors::Forbidden);
    }

    let existing_ritualist_data = RitualCharacterData::get_all_ritualists(&db_connection)
        .await
        .map_err(|err| {
            eprintln!("[RITUAL UPDATE DATA] Failed to read ritualist data! {err:?}");
            RootErrors::InternalServerError
        })?;

    // Let's build one big transaction so we don't have a bunch of inbetween moments.
    let sql_transaction = db_connection.transaction().await.map_err(|err| {
        eprintln!("[RITUAL UPDATE DATA] Errored trying to create an SQL Transaction! {err:?}");
        RootErrors::InternalServerError
    })?;

    let ids_in_existing_data: Vec<_> = existing_ritualist_data
        .iter()
        .map(|ritualist| ritualist.page_character.base_character.db_id)
        .collect();

    let ids_passed_by_user: Vec<_> = ritualist_data
        .iter()
        .map(|ritualist| ritualist.character_id)
        .collect();

    let deleted_character_ids: Vec<_> = ids_in_existing_data
        .iter()
        .filter(|id| !ids_passed_by_user.contains(id))
        .collect();

    let added_characters: Vec<_> = ritualist_data
        .iter()
        .filter(|ritualist| !ids_in_existing_data.contains(&ritualist.character_id))
        .collect();

    let modified_characters: Vec<_> = ritualist_data
        .iter()
        .filter(|ritualist| {
            existing_ritualist_data.iter().any(|existing_data| {
                existing_data.page_character.base_character.db_id == ritualist.character_id
                    && ((ritualist.power_description != existing_data.power_description)
                        || (ritualist.power_name == existing_data.power_name))
            })
        })
        .collect();

    for deleted_id in deleted_character_ids {
        // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
        let query = "DELETE FROM ritual_info WHERE character_id=$1;".to_string();

        sql_transaction
            .execute(&query, &[deleted_id])
            .await
            .map_err(|err| {
                eprintln!("[RITUAL UPDATE DATA] Adding SQL delete query failed! {err:?}");
                RootErrors::InternalServerError
            })?;

        println!(
                "[RITUAL UPDATE DATA] User {} (ID:{}) is attempting to delete ritual info on character with id {}. Proceeding with transaction.",
                requesting_user.display_name, requesting_user.id, deleted_id,
            );
    }

    for added_character_data in added_characters {
        let mut columns: Vec<String> = Vec::new();
        let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

        // TODO: Validate no one passed invalid character IDs. Right now it'll just return 500.
        columns.push("character_id".into());
        values.push(&added_character_data.character_id);

        columns.push("power_name".into());
        values.push(&added_character_data.power_name);

        columns.push("power_description".into());
        values.push(&added_character_data.power_description);

        // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
        let query = format!(
            "INSERT INTO ritual_info ({}) VALUES ({});",
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
                eprintln!("[RITUAL UPDATE DATA] Adding SQL insert query failed! {err:?}");
                RootErrors::InternalServerError
            })?;

        println!(
                "[RITUAL UPDATE DATA] User {} (ID:{}) is attempting to add ritual info on character with id {}. Proceeding with transaction.",
                requesting_user.display_name, requesting_user.id, added_character_data.character_id,
            );
    }

    for modified_character in modified_characters {
        let mut columns: Vec<String> = Vec::new();
        let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

        let existing_character_data = existing_ritualist_data
            .iter()
            .find(|existing_data| {
                existing_data.page_character.base_character.db_id == modified_character.character_id
            })
            .unwrap();

        if existing_character_data.power_description != modified_character.power_description {
            columns.push("power_description".into());
            values.push(&modified_character.power_description);
        }

        if existing_character_data.power_name != modified_character.power_name {
            columns.push("power_name".into());
            values.push(&modified_character.power_name);
        }

        if columns.is_empty() {
            continue;
        }

        values.push(&modified_character.character_id);

        // SAFETY: we're not inserting anything the user sent into the query. Everything user-inputted is passed as values later.
        let query = format!(
            "UPDATE ritual_info SET {} WHERE id=${};",
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
                eprintln!("[RITUAL UPDATE DATA] Adding SQL update query failed! {err:?}");
                RootErrors::InternalServerError
            })?;

        println!(
                "[RITUAL UPDATE DATA] User {} (ID:{}) is attempting to modify ritual info on character with id {}. Proceeding with transaction.",
                requesting_user.display_name, requesting_user.id, modified_character.character_id,
            );
    }

    // Now run it all and pray for the best.
    sql_transaction.commit().await.map_err(|err| {
        eprintln!("[RITUAL UPDATE DATA] Errored trying to run SQL Transaction! {err:?}");
        RootErrors::InternalServerError
    })?;

    println!(
        "[RITUAL UPDATE DATA] User {} (ID:{}) successfully modified ritual section.",
        requesting_user.display_name, requesting_user.id
    );
    Ok(http::StatusCode::CREATED.into_response())
}
