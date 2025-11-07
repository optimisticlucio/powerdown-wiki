use std::error::Error;
use std::path::Path;
use askama::Template;
use axum::extract::multipart::{Field};
use axum::extract::{Multipart, OriginalUri, State};
use axum::{Json, http};
use axum::response::{Html, IntoResponse, Response, Redirect};
use http::Uri;
use crate::art::structs::PageArt;
use crate::user::User;
use crate::utils::{template_to_response, compress_image_lossless, get_s3_object_url, text_or_internal_err};
use crate::{ServerState, errs::RootErrors};
use super::{structs::{BaseArt}};
use rand::{distr::Alphanumeric, Rng};
use std::io::Cursor;
use serde::{self, Deserialize, Serialize};

/// Post Request Handler for art category.
#[axum::debug_handler]
pub async fn add_art(State(state): State<ServerState>, Json(posting_step): Json<ArtPostingSteps>) -> Result<Response, RootErrors> {
    match posting_step {
        ArtPostingSteps::RequestPresignedURLs { art_amount } => {
            if art_amount < 1 {
                Err(RootErrors::BAD_REQUEST("art post must have at least one art piece".to_string()))
            } else if art_amount > 25 {
                Err(RootErrors::BAD_REQUEST("for the good of mankind, don't put that many art pieces in one post. split them up".to_string()))
            } else {
                let amount_of_presigned_urls_needed = art_amount + 1; // The art, plus the thumbnail.
                
                // TODO: Make this a vec<> of [amount_needed] random strings of sufficient length to not make problems.
                // TODO: Prefix "temp/art/" at the start of each string
                let mut temp_keys_for_presigned: Vec<String> = Vec::new();

                for key in &temp_keys_for_presigned {
                    // TODO: Open these keys as presigned in /temp/art/[key] in s3
                }

                // TODO: Prefix S3 URL on the start of amount_of_presigned_urls_needed

                // Send back the urls as a json.
                let response = serde_json::to_string(&PresignedUrlsResponse {
                    thumbnail_presigned_url: temp_keys_for_presigned.pop().unwrap(),
                    art_presigned_urls: temp_keys_for_presigned
                }).unwrap();

                Ok(response.into_response())
            }
        },
        ArtPostingSteps::UploadMetadata(page_art) => {
            // TODO: Validate all of the given values make sense.
            // TODO: Push to DB.

            Ok(Redirect::to(&format!("/art/{}", page_art.base_art.slug)).into_response())
        },
        _ => Err(RootErrors::BAD_REQUEST("invalid upload step".to_string()))
    }
}


/// Struct for reading the "steps" that a user (well, their client) needs to take to successfully upload art to the site.
#[derive(Deserialize)]
#[serde(tag = "step")]
pub enum ArtPostingSteps {
    #[serde(rename = "1")]
    RequestPresignedURLs {
        art_amount: u8 // It shouldn't be any bigger than *25* and positive. even u8 is overkill.
    },

    #[serde(rename="2")] 
    UploadMetadata(PageArt),
}

#[derive(Serialize)]
struct PresignedUrlsResponse {
    thumbnail_presigned_url: String,
    art_presigned_urls: Vec<String>,
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