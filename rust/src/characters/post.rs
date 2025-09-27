use axum::extract::multipart::Field;
use axum::extract::{multipart, Multipart, State};
use axum::response::{Html, IntoResponse};
use crate::{ServerState, characters::structs::{PageCharacterBuilder, BaseCharacterBuilder, InfoboxRow}, errs::RootErrors};

pub async fn add_character(State(state): State<ServerState>, mut multipart: Multipart) -> Result<impl IntoResponse, RootErrors> {
    let mut page_character_builder = PageCharacterBuilder::default();
    let mut base_character_builder = BaseCharacterBuilder::default();

    // TODO: All the unwraps here are BAD. These will easily crash the server!!
    while let Some(recieved_field) = multipart.next_field().await.unwrap() {
        let field_name = recieved_field.name().unwrap().to_string();
        
        match field_name.as_str() {
            // TODO: Missing values to recieve: Tag, long_name, logo_url, custom_css
            "name" => { base_character_builder.name(text_or_internal_err(recieved_field).await?); }
            "slug" => { base_character_builder.slug(text_or_internal_err(recieved_field).await?); }
            "thumbnail_url" => { base_character_builder.thumbnail_url(text_or_internal_err(recieved_field).await?); }
            "subtitles" => { 
                let field_text = text_or_internal_err(recieved_field).await?;
                let subtitle_array: Vec<String> = serde_json::from_str(&field_text).map_err(|parse_err| RootErrors::BAD_REQUEST(parse_err.to_string()))?;
                page_character_builder.subtitles(subtitle_array);
            }
            "creator" => { page_character_builder.creator(text_or_internal_err(recieved_field).await?); }
            "page_img_url" => { page_character_builder.page_img_url(text_or_internal_err(recieved_field).await?); }
            "infobox" => { 
                let field_text = text_or_internal_err(recieved_field).await?;
                let infobox_array: Vec<(String, String)> = serde_json::from_str(&field_text).map_err(|parse_err| RootErrors::BAD_REQUEST(parse_err.to_string()))?;
                page_character_builder.infobox(infobox_array.iter().map(|(title, desc)| InfoboxRow::new(title.to_owned(), desc.to_owned())).collect());
            }
            "overlay_css" => {
                page_character_builder.overlay_css(Some(text_or_internal_err(recieved_field).await?));
            }
            "page_contents" => {
                page_character_builder.page_contents(text_or_internal_err(recieved_field).await?);
            }
            "archival_reason" => {
                page_character_builder.archival_reason(Some(text_or_internal_err(recieved_field).await?));
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
            
            _ => return Err(RootErrors::BAD_REQUEST(format!("Invalid Field Recieved: {}", field_name)))
        }
    }

    let base_character = base_character_builder.build()
        .map_err(|err| RootErrors::BAD_REQUEST(err.to_string()))?;

    page_character_builder.base_character(base_character);
    let page_character = page_character_builder.build()
        .map_err(|err| RootErrors::BAD_REQUEST(err.to_string()))?;

    // TODO: Send character to DB.
    Ok(Html(format!("{} successfully recieved! Now start making code to put them in the DB.", &page_character.base_character.name)))
}

async fn text_or_internal_err(field: Field<'_>) -> Result<String, RootErrors> {
    field.text().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)
}