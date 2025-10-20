use std::error::Error;
use std::path::Path;
use askama::Template;
use axum::extract::multipart::{Field};
use axum::extract::{Multipart, OriginalUri, State};
use axum::http;
use axum::response::{Html, IntoResponse, Redirect};
use http::Uri;
use crate::user::User;
use crate::utils::{template_to_response, compress_image_lossless, get_s3_object_url};
use crate::{ServerState, errs::RootErrors};
use super::{structs::{BaseArtBuilder, PageArtBuilder, BaseArt}};
use rand::{distr::Alphanumeric, Rng};
use std::io::Cursor;

/// Post Request Handler for art category.
pub async fn add_art(State(state): State<ServerState>, mut multipart: Multipart) -> Result<impl IntoResponse, RootErrors> {
    let mut base_art_builder = BaseArtBuilder::default();
    let mut page_art_builder= PageArtBuilder::default();
    let mut art_url_collector = Vec::<(i32, String)>::new(); // (index, url). May be unordered.

    // TODO: Need to clean up temp art if the upload dropped.
    let temp_art_id: i32 = BaseArt::get_unused_id(state.db_pool.get().await.unwrap()).await;

    while let Some(recieved_field) = multipart.next_field().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)? {
        let field_name = match recieved_field.name() {
            None => return Err(RootErrors::BAD_REQUEST("Recieved a field with no name.".to_owned())),
            Some(x) => x
        };
        
        match field_name {
            _ if field_name.starts_with("file_") => {
                let file_index: i32 = field_name.strip_prefix("file_").unwrap()
                        .parse().map_err(|_| RootErrors::BAD_REQUEST(format!("Given invalid index in {}", field_name)))?;
                
                // Max filesize is rn 50mb. We can collect the whole thing at once for now, but later we should prob multipart it into s3 if it's above some file size.
                let user_given_file_extension = Path::new(recieved_field.file_name()
                    .ok_or(RootErrors::BAD_REQUEST(format!("File number {} lacked filename", file_index)))?).extension().unwrap().to_str().unwrap().to_string();

                // Possible to cause overlaps, practically unlikely.
                let random_art_slug: String = rand::rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect();

                let s3_file_name = format!("art/{}/{}.{}", temp_art_id, random_art_slug, &user_given_file_extension);
                let given_file = recieved_field.bytes().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

                // Check if we can compress the file.
                let compressed_given_file = {
                        // If is image, compress it.
                        if image::ImageReader::new(Cursor::new(&given_file))
                                .with_guessed_format().is_ok() {
                            compress_image_lossless(given_file.to_vec(), Some(&user_given_file_extension))
                                .unwrap_or(given_file.to_vec())
                        }   
                        else {
                            // In any other case, return self.
                            given_file.to_vec()
                        }
                };

                state.s3_client.put_object()
                        .bucket(&state.config.s3_public_bucket)
                        .key(&s3_file_name)
                        .body(compressed_given_file.into())
                        .send().await.map_err(|err| {
                            println!("INTERNAL ERROR! file upload for art temp_id {}", temp_art_id);
                            println!("Error: {}", err);
                            
                            // Print the full error chain
                            let mut source = err.source();
                            while let Some(e) = source {
                                println!("  Caused by: {}", e);
                                source = e.source();
                            }
                            
                            RootErrors::INTERNAL_SERVER_ERROR
                        })?; 
                
                art_url_collector.push((file_index, get_s3_object_url(&state.config.s3_public_bucket, &s3_file_name)));
            }
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
                let user_given_file_extension = Path::new(recieved_field.file_name()
                    .ok_or(RootErrors::BAD_REQUEST(format!("Thumbnail lacked filename")))?).extension().unwrap().to_str().unwrap().to_string();

                let s3_file_name = format!("art/{}/thumbnail.{}", temp_art_id, &user_given_file_extension);
                let given_file = recieved_field.bytes().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

                let compressed_given_file = compress_image_lossless(given_file.to_vec(), Some(&user_given_file_extension))
                                .unwrap_or(given_file.to_vec());

                state.s3_client.put_object()
                        .bucket(&state.config.s3_public_bucket)
                        .key(&s3_file_name)
                        .body(compressed_given_file.into())
                        .send().await.map_err(|err| {
                            println!("INTERNAL ERROR! thumbnail upload for art temp_id {}", temp_art_id);
                            println!("Error: {}", err);
                            
                            // Print the full error chain
                            let mut source = err.source();
                            while let Some(e) = source {
                                println!("  Caused by: {}", e);
                                source = e.source();
                            }
                            
                            RootErrors::INTERNAL_SERVER_ERROR
                        })?; 
                
                base_art_builder.thumbnail_url(get_s3_object_url(&state.config.s3_public_bucket, &s3_file_name));
            }
            "tags" => {
                let sent_text = text_or_internal_err(recieved_field).await?;

                let tags = serde_json::from_str(&sent_text)
                    .map_err(|_| RootErrors::BAD_REQUEST("Recieved invalid tag list.".to_owned()))?;
                page_art_builder.tags(tags);
            }
            "is_nsfw" => {
                // If this was sent at all, assume it is true.
                base_art_builder.is_nsfw(true);
            }
            "description" => {
                page_art_builder.description(Some(text_or_internal_err(recieved_field).await?));
            }
            _ => return Err(RootErrors::BAD_REQUEST(format!("Invalid Field Recieved: {}", field_name)))
        }
    }

    // Transaction complete, let's get all the art and organize it.
    art_url_collector.sort();
    page_art_builder.art_urls(art_url_collector.iter().map(|(_, url)| url.to_string()).collect::<Vec<_>>());

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

    // Un-hide the temp art 
    columns.push("is_hidden".into());
    values.push(&false);

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

    columns.push("is_nsfw".into());
    values.push(&page_art.base_art.is_nsfw);

    if let Some(description) = &page_art.description {
        columns.push("description".into());
        values.push(description);
    }

    let query = format!("UPDATE art SET {} WHERE id={}",
            columns.iter().enumerate().map(|(i, column_name)| format!("{}=${}",column_name, i+1)).collect::<Vec<String>>().join(", "),
            temp_art_id
        );

    db_connection.execute(&query, &values).await.map_err(|err| {
        println!("[ART POST] Error in db query execution!\nQuery: {}\nError: {:?}", query, err);
        RootErrors::INTERNAL_SERVER_ERROR
    })?;

    for (index, art_url) in page_art.art_urls.iter().enumerate() {
        let query = format!("INSERT INTO art_file(belongs_to,file_url,internal_order) VALUES($1,$2,$3)");

        // This cast is unsafe. However, if someone uploads an amount of art that can cause a 32bit stack overflow, I am personally
        // going to their house and having a fun conversation with them.
        db_connection.execute(&query, &[&temp_art_id, &art_url, &(index as i32)]).await.map_err(|err| {
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
    user: Option<User>,
    original_uri: Uri,
}

pub async fn art_posting_page(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    ) -> Result<impl IntoResponse, RootErrors> {
    Ok (
        template_to_response(
            ArtPostingPage {
                user: None, //TODO: Connect with user system.
                original_uri
            }
        )
    )
}