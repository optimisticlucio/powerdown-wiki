use std::collections::{HashMap};
use axum::extract::multipart::{Field, InvalidBoundary};
use axum::extract::{Multipart, State};
use axum::response::{Html, IntoResponse};
use crate::{ServerState, characters::structs::{PageCharacterBuilder, BaseCharacterBuilder, InfoboxRow}, errs::RootErrors};

pub async fn add_character(State(state): State<ServerState>, mut multipart: Multipart) -> Result<impl IntoResponse, RootErrors> {
    let mut page_character_builder = PageCharacterBuilder::default();
    let mut base_character_builder = BaseCharacterBuilder::default();

    // TODO: All the unwraps here are BAD. These will easily crash the server!!
    while let Some(recieved_field) = multipart.next_field().await.unwrap() {
        let field_name = recieved_field.name().unwrap().to_string();
        
        match field_name.as_str() {
            "name" => { base_character_builder.name(text_or_internal_err(recieved_field).await?); }
            "slug" => { base_character_builder.slug(text_or_internal_err(recieved_field).await?); }
            "thumbnail_url" => { base_character_builder.thumbnail_url(text_or_internal_err(recieved_field).await?); }
            "subtitles" => { 
                let field_text = text_or_internal_err(recieved_field).await?;
                let subtitle_array: Vec<String> = serde_json::from_str(&field_text).map_err(|parse_err| RootErrors::BAD_REQUEST(format!("{}, SUBTITLES, RECIEVED: {}",parse_err.to_string(), field_text)))?;
                page_character_builder.subtitles(subtitle_array);
            }
            "creator" => { page_character_builder.creator(text_or_internal_err(recieved_field).await?); }
            "page_img_url" => { page_character_builder.page_img_url(text_or_internal_err(recieved_field).await?); }
            "infobox" => { 
                let field_text = text_or_internal_err(recieved_field).await?;
                let infobox_array: HashMap<String, String> = serde_json::from_str(&field_text).map_err(|parse_err| RootErrors::BAD_REQUEST(format!("{}, INFOBOX, RECIEVED: {}",parse_err.to_string(), field_text)))?;
                page_character_builder.infobox(infobox_array.iter().map(|(title, desc)| InfoboxRow::new(title.to_owned(), desc.to_owned())).collect());
            }
            "overlay_css" => {
                page_character_builder.overlay_css(Some(text_or_internal_err(recieved_field).await?));
            }
            "page_contents" => {
                page_character_builder.page_contents(Some(text_or_internal_err(recieved_field).await?));
            }
            "retirement_reason" => {
                page_character_builder.retirement_reason(Some(text_or_internal_err(recieved_field).await?));
                base_character_builder.is_archived(true);
            }
            "is_main_character" => {
                // If this field is included at all, we assume it's true.
                base_character_builder.is_main_character(true);
            }
            "is_hidden" => {
                // If this field is included at all, we assume it's true.
                base_character_builder.is_hidden(true);
            }
            "relevant_tag" => {
                page_character_builder.tag(Some(text_or_internal_err(recieved_field).await?));
            }
            "logo" => {
                page_character_builder.logo_url(Some(text_or_internal_err(recieved_field).await?));
            }
            "long_name" => {
                page_character_builder.long_name(Some(text_or_internal_err(recieved_field).await?));
            }
            
            _ => return Err(RootErrors::BAD_REQUEST(format!("Invalid Field Recieved: {}", field_name)))
        }
    }

    let base_character = base_character_builder.build()
        .map_err(|err| RootErrors::BAD_REQUEST(err.to_string()))?;

    page_character_builder.base_character(base_character);
    let page_character = page_character_builder.build()
        .map_err(|err| RootErrors::BAD_REQUEST(err.to_string()))?;

    // Now the character is ready to send to the DB.
    let db_connection = state.db_pool.get().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

    // Let's build our query.
    let mut columns: Vec<String> = Vec::new();
    let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

    columns.push("page_slug".into());
    values.push(&page_character.base_character.slug);

    columns.push("short_name".into());
    values.push(&page_character.base_character.name);

    columns.push("subtitles".into());
    values.push(&page_character.subtitles);

    columns.push("creator".into());
    values.push(&page_character.creator);

    columns.push("thumbnail".into());
    values.push(&page_character.base_character.thumbnail_url);

    columns.push("page_image".into());
    values.push(&page_character.page_img_url);

    columns.push("infobox".into());
    values.push(&page_character.infobox);

    if let Some(overlay_css) = &page_character.overlay_css {
        columns.push("overlay_css".into());
        values.push(overlay_css);
    }

    if let Some(page_text) = &page_character.page_contents {
        columns.push("page_text".into());
        values.push(page_text);
    }

    if page_character.base_character.is_hidden {
        columns.push("is_hidden".into());
        values.push(&true);
    }

    if let Some(retirement_reason) = &page_character.retirement_reason {
        columns.push("retirement_reason".into());
        values.push(retirement_reason);
    }

    if let Some(long_name) = &page_character.long_name {
        columns.push("long_name".into());
        values.push(long_name);
    }

    if let Some(logo) = &page_character.logo_url {
        columns.push("logo".into());
        values.push(logo);
    }

    let query = format!("INSERT INTO character({}) VALUES ({})",
            columns.join(", "),
            columns.iter().enumerate().map(|(i, _)| format!("${}", i+1)).collect::<Vec<String>>().join(", "));

    db_connection.execute(&query, &values).await.map_err(|err| {
        println!("[CHARACTER POST] Error in db query execution!\nQuery: {}\nError: {:?}", query, err);
        RootErrors::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(format!("{} successfully recieved! Now start making code to put them in the DB.", &page_character.base_character.name)))
}

async fn text_or_internal_err(field: Field<'_>) -> Result<String, RootErrors> {
    field.text().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)
}