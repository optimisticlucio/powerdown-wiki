use std::collections::{HashMap};
use std::error::Error;
use std::path::Path;
use axum::extract::multipart::{Field, InvalidBoundary, MultipartError};
use axum::extract::{Multipart, State};
use axum::http;
use axum::response::{Html, IntoResponse, Redirect};
use regex::Regex;
use crate::characters::{page, BaseCharacter};
use crate::{ServerState, characters::structs::{PageCharacterBuilder, BaseCharacterBuilder, InfoboxRow}, errs::RootErrors};
use crate::utils::{self, get_s3_object_url, text_or_internal_err};

pub async fn add_character(State(state): State<ServerState>, mut multipart: Multipart) -> Result<impl IntoResponse, RootErrors> {
    let mut page_character_builder = PageCharacterBuilder::default();
    let mut base_character_builder = BaseCharacterBuilder::default();

    // TODO: Need to clean the temp character up if the upload dropped.
    let temp_character_id = BaseCharacter::get_unused_id(state.db_pool.get().await.unwrap()).await;

    while let Some(recieved_field) = multipart.next_field().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)? {
        let field_name = match recieved_field.name() {
            None => return Err(RootErrors::BAD_REQUEST("Recieved a field with no name.".to_owned())),
            Some(x) => x
        };

        match field_name {
            "name" => { 
                base_character_builder.name(text_or_internal_err(recieved_field).await?);
            }
            "slug" => { 
                let recieved_slug = text_or_internal_err(recieved_field)
                        .await?
                        .trim().to_owned();

                let check_slug_is_valid = Regex::new("^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();

                if !check_slug_is_valid.is_match(&recieved_slug) {
                    return Err(RootErrors::BAD_REQUEST("Slug has invalid formatting. Expecting lowercase, numbers, and single dashes between words.".to_owned()));
                }

                base_character_builder.slug(recieved_slug);
            }
            "thumbnail" => { 
                let user_given_file_extension = Path::new(recieved_field.file_name()
                    .ok_or(RootErrors::BAD_REQUEST("thumbnail lacked filename".to_string()))?).extension().unwrap().to_str().unwrap().to_string();

                let s3_file_name = format!("characters/{}/thumbnail.{}", temp_character_id, user_given_file_extension);
                let img_file = recieved_field.bytes().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

                let compressed_img_file = utils::compress_image_lossless(img_file.to_vec(), Some(&user_given_file_extension))
                    .unwrap_or(img_file.to_vec());

                let image_upload = state.s3_client.put_object()
                        .bucket(&state.config.s3_public_bucket)
                        .key(&s3_file_name)
                        .body(compressed_img_file.into())
                        .send().await.map_err(|err| {
                            println!("INTERNAL ERROR! thumbnail upload for temp_id char {}", temp_character_id);
                            println!("Error: {}", err);
                            
                            // Print the full error chain
                            let mut source = err.source();
                            while let Some(e) = source {
                                println!("  Caused by: {}", e);
                                source = e.source();
                            }
                            
                            RootErrors::INTERNAL_SERVER_ERROR
                        })?; 
                base_character_builder.thumbnail_url(get_s3_object_url(&state.config.s3_public_bucket, &s3_file_name));
            }
            "subtitles" => { 
                let field_text = text_or_internal_err(recieved_field).await?;
                let subtitle_array: Vec<String> = serde_json::from_str(&field_text).map_err(|parse_err| RootErrors::BAD_REQUEST(format!("{}, SUBTITLES",parse_err.to_string())))?;
                page_character_builder.subtitles(subtitle_array);
            }
            "creator" => { 
                page_character_builder.creator(text_or_internal_err(recieved_field).await?); 
            }
            "page_img" => { 
                let user_given_file_extension = Path::new(recieved_field.file_name()
                    .ok_or(RootErrors::BAD_REQUEST("page_img lacked filename".to_string()))?).extension().unwrap().to_str().unwrap().to_string();

                let s3_file_name = format!("characters/{}/page_img.{}", temp_character_id, user_given_file_extension);

                let img_file = recieved_field.bytes().await.map_err(|err| {
                    println!("[POST OF CHARACTER ID {}] - Failed to recieve page img: {}", &temp_character_id, &err.to_string());

                    RootErrors::BAD_REQUEST(err.body_text())
                })?;

                let compressed_img_file = utils::compress_image_lossless(img_file.to_vec(), Some(&user_given_file_extension))
                    .unwrap_or(img_file.to_vec());

                let image_upload = state.s3_client.put_object()
                        .bucket(&state.config.s3_public_bucket)
                        .key(&s3_file_name)
                        .body(compressed_img_file.into());
                
                image_upload.send().await.map_err(|err| {
                    println!("INTERNAL ERROR! thumbnail upload for temp_id char {}", temp_character_id);
                    println!("Error: {}", err);
                    
                    // Print the full error chain
                    let mut source = err.source();
                    while let Some(e) = source {
                        println!("  Caused by: {}", e);
                        source = e.source();
                    }
                    
                    RootErrors::INTERNAL_SERVER_ERROR
                })?; 

                page_character_builder.page_img_url(get_s3_object_url(&state.config.s3_public_bucket, &s3_file_name));
            }
            "infobox" => { 
                let field_text = text_or_internal_err(recieved_field).await?;
                let infobox_array: HashMap<String, String> = serde_json::from_str(&field_text).map_err(|parse_err| RootErrors::BAD_REQUEST(format!("{}, INFOBOX",parse_err.to_string())))?;
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

            "birthday" => {
                let recieved_birthday = text_or_internal_err(recieved_field).await?;

                let birthday_components: Vec<&str> = recieved_birthday.split("-").collect();

                if birthday_components.len() != 2 || birthday_components[0].len() != 2 || birthday_components[1].len() != 2 {
                    return Err(RootErrors::BAD_REQUEST("Birthday not in the MM-DD format.".to_owned()))
                }

                let birthday_u32 = birthday_components.iter().map(|date| date.parse::<u32>())
                    .collect::<Result<Vec<_>,_>>()
                    .map_err(|err| RootErrors::BAD_REQUEST("Birthday not in the MM-DD format.".to_owned()))?;

                let birthday = chrono::NaiveDate::from_ymd_opt(0, birthday_u32[0], birthday_u32[1])
                        .ok_or(RootErrors::BAD_REQUEST("Given a nonexistent date as a birthday.".to_owned()))?;

                base_character_builder.birthday(Some(birthday));
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
    let db_connection = state.db_pool.get().await.map_err(|_| {
        println!("CHARACTER POST ERR - Couldn't get DB connection for {}", &page_character.base_character.name);
        RootErrors::INTERNAL_SERVER_ERROR
    })?;

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
        // TODO: SANITIZE
        columns.push("page_text".into());
        values.push(page_text);
    }

    columns.push("is_hidden".into());
    values.push(&page_character.base_character.is_hidden);

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

    if let Some(birthday) = &page_character.base_character.birthday {
        columns.push("birthday".into());
        values.push(birthday);
    }

    if let Some(tag) = &page_character.tag {
        columns.push("relevant_tag".into());
        values.push(tag);
    }

    // Because we created a temp character, we'll update their values instead of creating a new one.
    let query = format!("UPDATE character SET {} WHERE id={}",
            columns.iter().enumerate().map(|(i, column_name)| format!("{}=${}",column_name, i+1)).collect::<Vec<String>>().join(", "),
            temp_character_id
        );

    db_connection.execute(&query, &values).await.map_err(|err| {
        println!("[CHARACTER POST] Error in db query execution!\nQuery: {}\nError: {:?}", query, err);
        RootErrors::INTERNAL_SERVER_ERROR
    })?;

    Ok(Redirect::to(&format!("/characters/{}", &page_character.base_character.slug)))
}
