use std::i16;

use askama::Template;
use axum::extract::multipart::{Field};
use axum::extract::{Multipart, State};
use axum::http;
use axum::response::{Html, IntoResponse, Redirect};
use crate::art;
use crate::user::User;
use crate::utils::{template_to_response, compress_image_lossless};
use crate::{ServerState, errs::RootErrors};
use super::{structs::{BaseArtBuilder, PageArtBuilder}};
use rayon::prelude::*;

/// Post Request Handler for art category.
pub async fn add_art(State(state): State<ServerState>, mut multipart: Multipart) -> Result<impl IntoResponse, RootErrors> {
    let mut base_art_builder = BaseArtBuilder::default();
    let mut page_art_builder= PageArtBuilder::default();

    while let Some(recieved_field) = multipart.next_field().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)? {
        let field_name = match recieved_field.name() {
            None => return Err(RootErrors::BAD_REQUEST("Recieved a field with no name.".to_owned())),
            Some(x) => x
        };
        
        match field_name {
            "slug" => {
                base_art_builder.slug(text_or_internal_err(recieved_field).await?);
            }
            "creation_date" => {
                let sent_date = text_or_internal_err(recieved_field).await?;

                // Expects date in ISO 8601 format (YYYY-MM-DD)
                let date = chrono::NaiveDate::parse_from_str(&sent_date, "%F")
                    .map_err(|_| RootErrors::BAD_REQUEST("Given invalid date. Please ensure your date was in the YYYY-MM-DD format.".to_owned()))?; 
                page_art_builder.creation_date(date);
            }
            "title" => {
                base_art_builder.title(text_or_internal_err(recieved_field).await?);
            }
            "creators" => {
                let sent_text = text_or_internal_err(recieved_field).await?;

                let creators: Vec<String> = serde_json::from_str(&sent_text)
                    .map_err(|_| RootErrors::BAD_REQUEST("Recieved invalid creator list.".to_owned()))?;
                base_art_builder.creators(creators);
            }
            "thumbnail" => {
                // TODO: Convert to file sending.
                base_art_builder.thumbnail_url(text_or_internal_err(recieved_field).await?);
            }
            "files" => {    
                // TODO: Convert to file sending.
                let sent_text = text_or_internal_err(recieved_field).await?;

                let files = serde_json::from_str(&sent_text)
                    .map_err(|_| RootErrors::BAD_REQUEST("Recieved invalid file list.".to_owned()))?;
                page_art_builder.art_urls(files);
            }
            "tags" => {
                let sent_text = text_or_internal_err(recieved_field).await?;

                let tags = serde_json::from_str(&sent_text)
                    .map_err(|_| RootErrors::BAD_REQUEST("Recieved invalid tag list.".to_owned()))?;
                page_art_builder.tags(tags);
            }
            "nsfw" => {
                // If this was sent at all, assume it is true.
                base_art_builder.nsfw(true);
            }
            "description" => {
                page_art_builder.description(Some(text_or_internal_err(recieved_field).await?));
            }
            _ => return Err(RootErrors::BAD_REQUEST(format!("Invalid Field Recieved: {}", field_name)))
        }
    }

    let base_art = base_art_builder.build()
        .map_err(|err| RootErrors::BAD_REQUEST(err.to_string()))?;
    page_art_builder.base_art(base_art);

    let page_art = page_art_builder.build()
        .map_err(|err| RootErrors::BAD_REQUEST(err.to_string()))?;

    // Now the art is ready to send to the DB.
    let db_connection = state.db_pool.get().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

    // Let's build our query.
    let mut columns: Vec<String> = Vec::new();
    let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

    columns.push("page_slug".into());
    values.push(&page_art.base_art.slug);

    columns.push("creation_date".into());
    values.push(&page_art.creation_date);

    columns.push("title".into());
    values.push(&page_art.base_art.title);

    columns.push("creators".into());
    values.push(&page_art.base_art.creators);

    columns.push("thumbnail".into());
    values.push(&page_art.base_art.thumbnail_url);

    columns.push("tags".into());
    values.push(&page_art.tags);

    columns.push("nsfw".into());
    values.push(&page_art.base_art.nsfw);

    if let Some(description) = &page_art.description {
        columns.push("description".into());
        values.push(description);
    }

    let query = format!("INSERT INTO art({}) VALUES ({}) RETURNING id",
            columns.join(", "),
            columns.iter().enumerate().map(|(i, _)| format!("${}", i+1)).collect::<Vec<String>>().join(", "));

    let new_art_id: i32 = db_connection.query_one(&query, &values).await.map_err(|err| {
        println!("[ART POST] Error in db query execution!\nQuery: {}\nError: {:?}", query, err);
        RootErrors::INTERNAL_SERVER_ERROR
    })?.get(0);

    for (index, art_url) in page_art.art_urls.iter().enumerate() {
        let query = format!("INSERT INTO art_file(belongs_to,file_url,internal_ordering) VALUES($1,$2,$3)");

        // This cast is unsafe. However, if someone uploads an amount of art that can cause a 32bit stack overflow, I am personally
        // going to their house and having a fun conversation with them.
        db_connection.execute(&query, &[&new_art_id, &art_url, &(index as i32)]).await.map_err(|err| {
            println!("[ART POST] Error in db query execution!\nQuery: {}\nError: {:?}", query, err);
            RootErrors::INTERNAL_SERVER_ERROR
        })?;
    }

    Ok(Redirect::to(&format!("/art/{}", &page_art.base_art.slug)))
}

async fn text_or_internal_err(field: Field<'_>) -> Result<String, RootErrors> {
    field.text().await
    .map_err(|err| match err.status() {
        http::status::StatusCode::BAD_REQUEST => RootErrors::BAD_REQUEST(err.body_text()),
        
        _ => RootErrors::INTERNAL_SERVER_ERROR
    })
}

#[derive(Template)] 
#[template(path = "art/post.html")]
struct ArtPostingPage {
    user: Option<User>
}

pub async fn art_posting_page(State(state): State<ServerState>) -> Result<impl IntoResponse, RootErrors> {
    Ok (
        template_to_response(
            ArtPostingPage {
                user: None //TODO: Connect with user system.
            }
        )
    )
}