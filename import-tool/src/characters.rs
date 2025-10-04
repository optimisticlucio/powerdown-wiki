use std::{fs, path::{Path, PathBuf}, sync::Arc};
use reqwest::{Url};
use serde::{Deserialize, Serialize};
use gray_matter::{Matter, engine::YAML};
use indexmap::IndexMap;
use owo_colors::{ OwoColorize};
use futures::{stream, StreamExt};
use tokio::sync::Mutex;
use indicatif::ProgressBar;

pub async fn select_import_options(root_path: &Path, server_url: &Url) {
    // TODO: Find _characters folder, get all files within it.
    let all_character_paths = Vec::<PathBuf>::new();

    // TODO: Show user amount of characters in folder.

    println!("Would you like to\n{}\n{}\nor {}?\n{}", 
        "(1) Import all characters".yellow(), 
        "(2) Import a random group of characters".blue(), 
        "(3) Import a specific file".green(), 
        "Press 0 to exit screen.".italic());

    loop {
        let chosen_option = crate::read_line().unwrap();

        let trimmed_option = chosen_option.trim();

        match trimmed_option {
            "1" => { // Import all
                if let Err(import_errs) = import_given_characters(&all_character_paths, server_url).await {
                    println!("---{}---\n{}\n------", "There were errors during the import!".red(), import_errs.join("\n"))
                }
                else {
                    println!("{}", "Import completed without problems!".green())
                }
                break;
            }

            "2" => { // TODO: Import X randomly
                unimplemented!();
                break;
            }

            "3" => { // TODO: Import specific file
                unimplemented!();
                break;
            }

            "0" => {
                break;
            }
            _ => println!("{}", "I didn't quite get that.".yellow())
        }
    }

    unimplemented!();
}

async fn import_given_characters(character_file_paths: &Vec<PathBuf>, server_url: &Url) -> Result<(), Vec<String>> {
    let simultaneous_threads = 4;
    let import_errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // TODO: If try_into doesn't work, use a spinner.
    let progress_bar = Arc::new(ProgressBar::new(character_file_paths.len().try_into().unwrap()).with_finish(indicatif::ProgressFinish::AndClear));

    stream::iter(character_file_paths)
                    .for_each_concurrent(simultaneous_threads, |character_path| {
                        let import_errors_clone = import_errors.clone();
                        let progress_bar_clone = progress_bar.clone();

                        async move {
                            let import_result = import_given_character(character_path, server_url).await;

                            if let Err(import_error) = import_result {
                                let mut import_errors_unlocked = import_errors_clone.blocking_lock();
                                import_errors_unlocked.push(import_error);
                            }

                            progress_bar_clone.inc(1);
                        }
                    }).await;
    
    let errors =  {
        // TODO: Handle if somehow this mutex wasn't released.
        let errors_mutex_lock = import_errors.blocking_lock();
        errors_mutex_lock.clone()
    };

    if errors.is_empty() {
        Ok(())
    }
    else {
        Err(errors)
    }
}

async fn import_given_character(character_file_path: &Path, server_url: &Url) -> Result<(), String> {
    // Read and parse file
    let file_contents = fs::read_to_string(character_file_path).map_err(|err| format!("File Read Err: {}", err.to_string()))?;

    let parser = Matter::<YAML>::new();
    let parsed_file = parser.parse(&file_contents).map_err(|err| format!("File Parse Err: {}", err.to_string()))?;
    
    let frontmatter: CharacterFrontmatter = parsed_file.data.ok_or("File Parse Err: No Frontmatter")?;
    let file_content = parsed_file.content;
    let character_slug: String = character_file_path.file_name().unwrap().to_ascii_lowercase().to_str().unwrap().trim_end_matches(".md").to_owned();

    // Set the required fields for the post request
    let post_request = reqwest::multipart::Form::new()
        .text("name", frontmatter.character_title)
        .text("creator", frontmatter.character_author)
        .text("slug", character_slug.clone())
        .text("thumbnail_url", format!("https://powerdown.wiki/assets/img/characters/thumbnails/{}", character_slug)) // TODO: Convert to file sending
        .text("page_img_url", format!("https://powerdown.wiki/assets/img/{}", frontmatter.character_img_file)) // TODO: Convert to file sending
        .text("subtitles", serde_json::to_string(&frontmatter.character_subtitle).map_err(|err| format!("Subtitle JSON Err: {}", err.to_string()))?)
        .text("infobox", serde_json::to_string(&frontmatter.infobox_data).map_err(|err| format!("Infobox JSON Err: {}", err.to_string()))?)
        .text("page_contents", file_content)
        ;

    // TODO: Implement optional fields

    // Send the post request and hope for the best.
    reqwest::Client::new().post(server_url.to_owned())
        .multipart(post_request).send()
        .await.map_err(|err| format!("Info Send Err: {}", err.to_string()))?
        ;

    Ok(())
}

#[derive(Deserialize, Serialize)]
struct CharacterFrontmatter {
    #[serde(rename = "exclusion-reason")]
    archival_reason: Option<String>, // convert to archival_reason
    #[serde(default)]
    #[serde(rename = "hide-character")]
    hide_character: bool, // If missing, assume false.
    #[serde(rename = "character-title")]
    character_title: String,
    #[serde(rename = "inpage-character-title")]
    inpage_character_title: Option<String>, // Convert to long_name
    character_subtitle: Vec<String>,
    character_author: String,
    #[serde(rename = "logo-file")]
    logo_file: Option<String>,
    #[serde(rename = "character-img-file")]
    character_img_file: String, // The way it's written is relative to /assets/img/. Account for that.
    birthday: String, // Written as MM-DD
    #[serde(rename = "infobox-data")]
    infobox_data: IndexMap<String, String>,
    // I'm dropping relationships, this feature sucks.
    #[serde(rename = "css-code")]
    overlay_css: Option<String>

    // TODO: Handle ritual stuff
}