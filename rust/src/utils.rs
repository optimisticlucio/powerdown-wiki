use std::error::Error;
use std::{env, fmt};

use chrono::{DateTime, Datelike, Utc};
use axum::response::{Response, IntoResponse, Html};
use axum::body::Body;
use askama::Template;
use crate::errs::RootErrors;
use serde::{Deserialize};
use serde::de::{Deserializer};
use axum::extract::multipart::{Field};

pub mod file_compression;

pub use file_compression::compress_image_lossless;

pub fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
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
    let readable_day = match date.day() {
        1 => "1st".to_owned(),
        2 => "2nd".to_owned(),
        3 => "3rd".to_owned(),
        x => format!("{x}th")
    };
    let readable_month = date.format("%B");

    format!("{readable_month} {readable_day}")
}

pub fn join_names_human_readable(names: Vec<&str>) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].to_string(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => format!("{}, and {}", names[..names.len()-1].join(", "), names.last().unwrap()),
    }
}

pub fn template_to_response<T: Template>(template: T) -> Response<Body> {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            eprintln!("Failed to render template: {err:?}");
            RootErrors::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Returns the public-facing URL for an S3 object, given its key and bucket.
pub fn get_s3_object_url(bucket_name: &str, file_key: &str) -> String {
    format!("http://localhost:4566/{}/{}", bucket_name, file_key)
}

/// Returns the public-facing URL for an S3 object in the public bucket.
pub fn get_s3_public_object_url(file_key: &str) -> String {
    get_s3_object_url(&env::var("S3_PUBLIC_BUCKET_NAME").unwrap(), file_key)
}

pub async fn text_or_internal_err(field: Field<'_>) -> Result<String, RootErrors> {
    field.text().await
    .map_err(|err| match err.status() {
        http::status::StatusCode::BAD_REQUEST => RootErrors::BAD_REQUEST(http::Uri::from_static("/"), tower_cookies::Cookies::default(), err.body_text()),
        _ => RootErrors::INTERNAL_SERVER_ERROR
    })
}

/// Given a file on the public bucket, attempts to optimize it and move it to the target bucket under the target key.
/// Mainly for usage with temp images uploaded by users.
pub async fn move_temp_s3_file(
        s3_client: aws_sdk_s3::Client,
        server_config: &crate::server_state::config::Config,
        temp_file_key: &str,
        target_bucket_name: &str,
        target_file_key: &str
    ) -> Result<(), MoveTempS3FileErrs> {

    let downloaded_file = s3_client.get_object()
        .bucket(&server_config.s3_public_bucket)
        .key(temp_file_key)
        .send()
        .await
        .map_err(|err| {
            eprintln!("[MOVE TEMP S3 FILE] Failed to move temp file {temp_file_key} to target {target_file_key} due to an error during download: {}", err.to_string());
            MoveTempS3FileErrs::DownloadFailed
        })?;

    let content_type = downloaded_file.content_type().map(str::to_string);

    let original_file_bytes = downloaded_file.body
        .collect().await.map_err(|err| {
            eprintln!("[MOVE TEMP S3 FILE] Failed to move temp file {temp_file_key} to target {target_file_key} due to an error in byte collection: {}", err.to_string());
            MoveTempS3FileErrs::ConversionFailed
        })?
        .into_bytes().to_vec();

    let converted_file = file_compression::compress_image_lossless(original_file_bytes.to_vec(), content_type.as_deref())
        .unwrap_or(original_file_bytes.to_vec()); // If can't compress it, just send back the original untouched.

    s3_client.put_object()
        .bucket(target_bucket_name)
        .key(target_file_key)
        .body(converted_file.into())
        .send()
        .await
        .map_err(|err| {
            eprintln!("[MOVE TEMP S3 FILE] Failed to move temp file {temp_file_key} to target {target_file_key} due to an error during upload: {}", err.to_string());
            MoveTempS3FileErrs::UploadFailed
        })?;

    Ok(())
}

#[derive(Debug)]
pub enum MoveTempS3FileErrs {
    DownloadFailed,
    ConversionFailed,
    UploadFailed
}

impl fmt::Display for MoveTempS3FileErrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DownloadFailed => write!(f, "Download Failed"),
            Self::ConversionFailed => write!(f, "Conversion Failed"),
            Self::UploadFailed => write!(f, "Upload Failed")
        }
    }
}

impl Error for MoveTempS3FileErrs {

}
