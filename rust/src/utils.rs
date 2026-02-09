use std::error::Error;
use std::{env, fmt};

use crate::errs::RootErrors;
use crate::ServerState;
use askama::Template;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::types::ObjectIdentifier;
use axum::body::Body;
use axum::response::{Html, IntoResponse, Response};
use chrono::{DateTime, Datelike, Utc};
use http::Uri;
use rand::distr::{Alphanumeric, SampleString};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;

pub mod arbitrary_values;
pub mod file_compression;
pub mod sql;

#[allow(dead_code)] // This is used by serde multiple times in the app, but the compiler can't tell. Don't delete this, jackass.
pub fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Single(String),
        Multiple(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::Single(s) => Ok(vec![s]),
        StringOrVec::Multiple(v) => Ok(v),
    }
}

pub fn format_date_to_human_readable(date: DateTime<Utc>) -> String {
    let day_number = date.day();

    let readable_day = if (11..13).contains(&day_number) {
        // Handling "11th, 12th, 13th" first.
        format!("{day_number}th")
    } else {
        match day_number % 10 {
            1 => format!("{day_number}st"),
            2 => format!("{day_number}nd"),
            3 => format!("{day_number}rd"),
            _ => format!("{day_number}th"),
        }
    };
    let readable_month = date.format("%B");

    format!("{readable_month} {readable_day}")
}

pub fn join_names_human_readable(names: Vec<&str>) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].to_string(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => format!(
            "{}, and {}",
            names[..names.len() - 1].join(", "),
            names.last().unwrap()
        ),
    }
}

pub fn template_to_response<T: Template>(template: T) -> Response<Body> {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            eprintln!("Failed to render template: {err:?}");
            RootErrors::InternalServerError.into_response()
        }
    }
}

// TODO: This is a hotpath, there's gotta be a better way to do this shit.
/// Returns the public-facing URL for an S3 object, given its key and bucket.
pub fn get_s3_object_url(bucket_name: &str, file_key: &str) -> String {
    // Check if we have an explicit public-facing URL base (for LocalStack or CloudFront)
    if let Ok(base_url) = env::var("S3_PUBLIC_FACING_URL") {
        format!("{base_url}/{bucket_name}/{file_key}")
    } else {
        let region = env::var("AWS_REGION").unwrap();
        format!("https://{bucket_name}.s3.{region}.amazonaws.com/{file_key}",)
    }
}

// TODO: This is a hotpath, there's gotta be a better way to do this.
/// Returns the public-facing URL for an S3 object in the public bucket.
pub fn get_s3_public_object_url(file_key: &str) -> String {
    if let Ok(public_bucket_url) = env::var("S3_PUBLIC_BUCKET_URL") {
        format!("{public_bucket_url}/{file_key}")
    } else {
        get_s3_object_url(&env::var("S3_PUBLIC_BUCKET_NAME").unwrap(), file_key)
    }
}

/// Given a file on the public bucket, attempts to optimize it and move it to the target bucket under the target key.
/// Returns the key that it was uploaded to (with the file extension).
/// Mainly for usage with temp images uploaded by users.
pub async fn move_temp_s3_file(
    s3_client: &aws_sdk_s3::Client,
    server_config: &crate::server_state::config::Config,
    temp_file_key: &str,
    target_bucket_name: &str,
    target_file_key: &str,
) -> Result<String, MoveTempS3FileErrs> {
    fn losslessly_convert_based_on_filetype(
        original_file_bytes: Vec<u8>,
        mime_type: &str,
        mime_media_type: &str,
    ) -> Option<Vec<u8>> {
        match mime_media_type {
            "image" if mime_type == "image/svg+xml" => Some(original_file_bytes), // God SVGs are a fuckin' headache.
            "image" => Some(
                file_compression::compress_image_lossless(original_file_bytes.to_vec(), mime_type)
                    .unwrap_or(original_file_bytes),
            ), // If can't compress it, just send back the original untouched.
            "video" => Some(original_file_bytes), // Video compression takes ages, I'm not doing it on-server.
            _ => None,
        }
    }

    move_and_convert_temp_file(
        s3_client,
        server_config,
        temp_file_key,
        target_bucket_name,
        target_file_key,
        losslessly_convert_based_on_filetype,
        "MOVE TEMP S3 FILE",
    )
    .await
}

/// Given an image on the public bucket, attempts to compress it and move it to the target bucket under the target key.
/// Returns the key that it was uploaded to (with the file extension).
/// If passed something that isn't an image, returns UnknownFiletype.
pub async fn move_and_lossily_compress_temp_s3_img(
    s3_client: &aws_sdk_s3::Client,
    server_config: &crate::server_state::config::Config,
    temp_file_key: &str,
    target_bucket_name: &str,
    target_file_key: &str,
    compression_settings: Option<file_compression::LossyCompressionSettings>,
) -> Result<String, MoveTempS3FileErrs> {
    fn lossily_compress_img(
        original_file_bytes: Vec<u8>,
        mime_type: &str,
        mime_media_type: &str,
        compression_settings: Option<file_compression::LossyCompressionSettings>,
    ) -> Option<Vec<u8>> {
        // If it's not an image, SHOOT THAT SHIT BACK.
        if mime_media_type != "image" {
            return None;
        }

        // If it's an SVG just skip this whole thing, it's as compressed as it'll get.
        if mime_type == "image/svg+xml" {
            return Some(original_file_bytes);
        }

        // Now it's gotta be an image. COMPRESS IT.
        file_compression::compress_image_lossy(original_file_bytes, mime_type, compression_settings)
            .ok()
    }

    move_and_convert_temp_file(
        s3_client,
        server_config,
        temp_file_key,
        target_bucket_name,
        target_file_key,
        move |x, y, z| lossily_compress_img(x, y, z, compression_settings),
        "COMPRESS TEMP S3 IMG",
    )
    .await
}

/// Helper function which downloads a temp file from the public bucket, runs a function on it, and moves it to a chosen final location in any bucket.
async fn move_and_convert_temp_file<F>(
    s3_client: &aws_sdk_s3::Client,
    server_config: &crate::server_state::config::Config,
    temp_file_key: &str,
    target_bucket_name: &str,
    target_file_key: &str,
    file_conversion_operation: F,
    function_name_for_debug_logging: &str,
) -> Result<String, MoveTempS3FileErrs>
where
    F: FnOnce(Vec<u8>, &str, &str) -> Option<Vec<u8>>,
{
    // Download file from S3
    let downloaded_file = s3_client.get_object()
        .bucket(&server_config.s3_public_bucket)
        .key(temp_file_key)
        .send()
        .await
        .map_err(|err| {
            eprintln!("[{function_name_for_debug_logging}] Failed to move temp file {temp_file_key} to target {target_file_key} due to an error during download: {err:?}");
            MoveTempS3FileErrs::DownloadFailed
        })?;

    let original_file_bytes = downloaded_file.body
        .collect().await.map_err(|err| {
            eprintln!("[{function_name_for_debug_logging}] Failed to move temp file {temp_file_key} to target {target_file_key} due to an error in byte collection: {err:?}");
            MoveTempS3FileErrs::ConversionFailed
        })?
        .into_bytes().to_vec();

    // Now let's find out what kind of file this is, and compress it appropriately.
    // TODO: This section is string-dependent. It works for now,
    // but there's gotta be some crate with a MIME type that can make this less error-prone. Rewrite later!
    let mut mime_type = infer::get(&original_file_bytes)
        .map(|file_type| file_type.mime_type())
        .or_else(|| {
            let detected = tree_magic_mini::from_u8(&original_file_bytes);
            if detected == "application/octet-stream" {
                // If we get octet stream it means tree_magic has no idea.
                None
            } else {
                Some(detected)
            }
        })
        .ok_or_else(|| {
            eprintln!(
                "[{function_name_for_debug_logging}] File key {temp_file_key} has an unknown filetype."
            );
            MoveTempS3FileErrs::UnknownFiletype
        })?;

    // For some reason, svgs isn't detected properly. This is an attempt to fix that.
    if mime_type == "text/plain" && is_an_svg(&original_file_bytes) {
        mime_type = "image/svg+xml";
    }

    let mime_media_type = mime_type.split("/").next().unwrap();
    let mime_type_extension = match mime_type {
        // Handle cases where mime_guess returns a weird file extension.
        "text/plain" => "txt",
        _ => mime_guess::get_mime_extensions_str(mime_type)
            .and_then(|exts| exts.first())
            .unwrap_or(&"bin"),
    };

    // Now run the relevant operation on the file.
    let converted_file =
        match file_conversion_operation(original_file_bytes, mime_type, mime_media_type) {
            Some(x) => x,
            None => {
                // If the inner function passed none, it's assumed it failed somehow.
                return Err(MoveTempS3FileErrs::ConversionFailed);
            }
        };

    // As far as I know, this is only referenced when the browser decides whether to display or download a file.
    // Inline displays in-browser, attach downloads it.
    let file_content_disposition = match mime_media_type {
        "image" | "video" | "audio" => "inline",
        _ => "attachment",
    };

    let target_key_with_filename = format!(
        "{}.{}",
        target_file_key.split(".").next().unwrap(), // Remove a passed extension.
        mime_type_extension
    );

    s3_client.put_object()
        .bucket(target_bucket_name)
        .key(&target_key_with_filename)
        .body(converted_file.into())
        .content_type(mime_type)
        .content_disposition(file_content_disposition)
        .send()
        .await
        .map_err(|err| {
            eprintln!("[{function_name_for_debug_logging}] Failed to move temp file {temp_file_key} to target {target_file_key} due to an error during upload: {err:?}");
            MoveTempS3FileErrs::UploadFailed
        })?;

    Ok(target_key_with_filename)
}

#[derive(Debug)]
pub enum MoveTempS3FileErrs {
    DownloadFailed,
    ConversionFailed,
    UploadFailed,
    UnknownFiletype,
}

impl fmt::Display for MoveTempS3FileErrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DownloadFailed => write!(f, "Download Failed"),
            Self::ConversionFailed => write!(f, "Conversion Failed"),
            Self::UploadFailed => write!(f, "Upload Failed"),
            Self::UnknownFiletype => write!(f, "Unknown Filetype"),
        }
    }
}

impl Error for MoveTempS3FileErrs {}

/// Struct for reading the "steps" that a user (well, their client) needs to take to successfully upload
/// various post types to the site, such as art or characters.
#[derive(Debug, Deserialize)]
#[serde(tag = "step")]
pub enum PostingSteps<T> {
    #[serde(rename = "1")]
    RequestPresignedURLs {
        #[serde(default)]
        file_amount: u8, // It shouldn't be any bigger than *25* and positive. even u8 is overkill.
    },

    #[serde(rename = "2")]
    UploadMetadata(T),
}

/// Returns a list of [amount_of_presigned_urls_needed] presigned URLs from S3.
pub async fn get_temp_s3_presigned_urls(
    state: &crate::ServerState,
    amount_of_presigned_urls_needed: u32,
    s3_temp_folder_name: &str,
) -> Result<Vec<String>, String> {
    let temp_key_tasks: Vec<_> = (0..amount_of_presigned_urls_needed)
        .map(|_| {
            let s3_client = state.s3_client.clone();
            let public_bucket_key = state.config.s3_public_bucket.clone();
            let s3_temp_folder_name = s3_temp_folder_name.to_string();

            tokio::spawn(async move {
                let random_key = Alphanumeric.sample_string(&mut rand::rng(), 64);
                let temp_art_key = format!("temp/{s3_temp_folder_name}/{random_key}");

                // get s3 to open a presigned URL for the temp key.
                s3_client
                    .put_object()
                    .bucket(public_bucket_key)
                    .key(temp_art_key)
                    .presigned(
                        PresigningConfig::expires_in(Duration::from_secs(300)).unwrap(), // Five minutes to upload. May be too much?
                    )
                    .await
                    .map(|x| x.uri().to_string())
            })
        })
        .collect();

    let mut temp_keys_for_presigned = Vec::new();

    for task in temp_key_tasks {
        let uri = task
            .await
            .map_err(|err| format!("Tokio Join Err! {err:?}"))?
            .map_err(|err| format!("[ART POST STAGE 1] SDK presigned URL creation err! {err:?}"))?;

        temp_keys_for_presigned.push(uri);
    }

    // In development (LocalStack), presigned URLs point to internal docker hostnames. Which is a problem.
    // In production this isn't a thing, so just leave S3_PUBLIC_FACING_URL unset.
    if let Ok(public_base) = env::var("S3_PUBLIC_FACING_URL") {
        let public_uri = Uri::from_str(&public_base).unwrap();

        temp_keys_for_presigned = temp_keys_for_presigned
            .iter()
            .map(|presigned_url| {
                let presigned_uri = Uri::from_str(presigned_url).unwrap();

                // Replace the scheme and authority, keep path and query (including signature params)
                Uri::builder()
                    .scheme(public_uri.scheme().unwrap().clone())
                    .authority(public_uri.authority().unwrap().clone())
                    .path_and_query(presigned_uri.path_and_query().unwrap().clone())
                    .build()
                    .unwrap()
                    .to_string()
            })
            .collect();
    }

    Ok(temp_keys_for_presigned)
}

#[derive(Debug, Serialize)]
/// A struct that should be sent as json to the user if they request presigned urls.
pub struct PresignedUrlsResponse {
    pub presigned_urls: Vec<String>,
}

/// Given a user-given key for the S3, returns Some(String) if the key was successfully cleaned. None if the key is empty or otherwise invalid.
pub fn clean_passed_key(passed_url: &str, state: &ServerState) -> Option<String> {
    let parsed_url = Uri::from_str(passed_url).ok()?;
    let key = parsed_url.path();

    if key.is_empty() {
        None
    } else {
        Some(
            key
                // To handle both [bucket_name].[domain]/[key] and [domain]/[bucket_name]/[key] cases
                .trim_start_matches(&format!("/{}", state.config.s3_public_bucket))
                .trim_start_matches("/")
                .to_string(),
        )
    }
}

/// Given a list of keys to delete from S3, and the bucket to delete them from, attempts a delete.
pub async fn delete_keys_from_s3(
    s3_client: &aws_sdk_s3::Client,
    bucket_to_delete_from: &str,
    keys_to_delete: &[String],
) -> Result<(), String> {
    if keys_to_delete.is_empty() {
        return Ok(());
    }

    let files_to_delete: Vec<ObjectIdentifier> = keys_to_delete
        .iter()
        .map(|key| ObjectIdentifier::builder().key(key).build().unwrap())
        .collect();

    s3_client
        .delete_objects()
        .bucket(bucket_to_delete_from)
        .delete(
            aws_sdk_s3::types::Delete::builder()
                .set_objects(Some(files_to_delete))
                .build()
                .unwrap(),
        )
        .send()
        .await
        .map_err(|err| format!("{err:?}"))?;

    Ok(())
}

/// Given a file, returns whether I think its an SVG. For some reason, this is a headache to do.
fn is_an_svg(file: &[u8]) -> bool {
    if let Ok(content) = std::str::from_utf8(file) {
        let content_lower = content.to_lowercase();
        let trimmed = content_lower.trim_start();

        // SVG files typically start with <?xml or <svg, and contain svg namespace
        (trimmed.starts_with("<?xml")
            || trimmed.starts_with("<svg")
            || trimmed.starts_with("<!doctype svg"))
            && content_lower.contains("<svg")
    } else {
        false
    }
}

/// Given a string, returns whether we'd accept it as a URL slug.
pub fn is_valid_slug(slug: &str) -> bool {
    let re = regex::Regex::new(r"^[a-z0-9]+(?:[-_][a-z0-9]+)*$").unwrap();
    re.is_match(slug)
}

/// Given a string, returns whether we'd accept it as a post tag.
pub fn is_valid_tag(tag: &str) -> bool {
    let re = regex::Regex::new(r"^[a-z0-9]+(?:[-_][a-z0-9]+)*$").unwrap();
    re.is_match(tag)
}
