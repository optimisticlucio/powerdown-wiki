use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Error as DeError};
use std::pin::Pin;
use std::{fs, path::{Path, PathBuf}, sync::Arc};
use tokio::sync::Mutex;
use indicatif::ProgressBar;
use indexmap::IndexMap;
use futures::{stream, StreamExt};
use reqwest::Response;

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

pub fn deserialize_string_map<'de, D>(deserializer: D) -> Result<IndexMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrOther {
        String(String),
        Number(serde_json::Number),
        Bool(bool),
    }
    
    let map: IndexMap<String, StringOrOther> = IndexMap::deserialize(deserializer)?;
    
    Ok(map
        .into_iter()
        .map(|(k, v)| {
            let string_value = match v {
                StringOrOther::String(s) => s,
                StringOrOther::Number(n) => n.to_string(),
                StringOrOther::Bool(b) => b.to_string(),
            };
            (k, string_value)
        })
        .collect())
}

pub async fn run_multiple_imports<'a, F, Fut>(
        root_path: &'a Path,
        file_paths: &'a Vec<PathBuf>,
        server_url: &'a Url,
        import_function: &'a F,
    ) -> Result<Vec<String>, Vec<String>>
    where
        F: Fn(&'a Path, &'a Path, &'a Url) -> Fut,
        Fut: Future<Output = Result<Response, String>> + 'a,
    {
    let simultaneous_threads = 8;
    let import_errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let import_successes: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let progress_bar = Arc::new(ProgressBar::new(file_paths.len().try_into().unwrap()).with_finish(indicatif::ProgressFinish::AndClear));

    stream::iter(file_paths)
                    .for_each_concurrent(simultaneous_threads, |current_path| {
                        let import_errors_clone = import_errors.clone();
                        let import_successes_clone = import_successes.clone();
                        let progress_bar_clone = progress_bar.clone();

                        async move {
                            match import_function(&root_path, current_path, server_url).await {
                                Err(mut import_error) => {  
                                    import_error.insert_str(0, &format!("{:?} ", current_path.file_name().unwrap()));
                                    let mut import_errors_unlocked = import_errors_clone.lock().await;
                                    import_errors_unlocked.push(import_error);
                                },
                                Ok(import_success) => {
                                    let import_success_readable = format!("{:?} STATUS {}: {}", current_path.file_name().unwrap(), import_success.status(), import_success.text().await.unwrap_or_default());
                                    let mut import_successes_unlocked = import_successes_clone.lock().await;
                                    import_successes_unlocked.push(import_success_readable);
                                }
                            }

                            progress_bar_clone.inc(1);
                        }
                    }).await;
    
    let mut errors =  {
        // TODO: Handle if somehow this mutex wasn't released.
        let errors_mutex_lock = import_errors.lock().await;
        errors_mutex_lock.clone()
    };

    let successes =  {
        // TODO: Handle if somehow this mutex wasn't released.
        let successes_mutex_lock = import_successes.lock().await;
        successes_mutex_lock.clone()
    };

    errors.extend(successes.iter().filter(|success| !success.contains("STATUS 200")).map(|x| x.to_owned()).collect::<Vec<String>>());

    if errors.is_empty() {
        Ok(successes)
    }
    else {
        Err(errors)
    }
}

pub async fn send_to_presigned_url(target_url: &str, file: Vec<u8>) -> Result<Response, reqwest::Error> {
    let corrected_url = if target_url.starts_with("http://host.docker.internal:4566/") {
        format!("http://localstack:4566/{}", target_url.trim_start_matches("http://host.docker.internal:4566/"))
    } else { target_url.to_string() };

    let content_type = if let Some(kind) = infer::get(&file) {
        kind.mime_type()
    } else {
        "application/octet-stream" // fallback
    };

    reqwest::Client::new()
        .put(&corrected_url)
        .header("Content-Type", content_type)
        .body(file)
        .send()
        .await
}

#[derive(Debug, Deserialize)]
/// A struct that should be sent as json to the user if they request presigned urls.
pub struct PresignedUrlsResponse {
    pub presigned_urls: Vec<String>,
}

/// Struct for reading the "steps" that a user (well, their client) needs to take to successfully upload
/// various post types to the site, such as art or characters.
#[derive(Debug, Serialize)]
#[serde(tag = "step")]
pub enum PostingSteps<T> {
    #[serde(rename = "1")]
    RequestPresignedURLs {
        #[serde(default)]
        art_amount: u8, // It shouldn't be any bigger than *25* and positive. even u8 is overkill.
    },

    #[serde(rename = "2")]
    UploadMetadata(T),
}