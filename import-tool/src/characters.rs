use std::{error::Error, fs, path::{Path, PathBuf}, sync::Arc};
use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use gray_matter::{Matter, engine::YAML};
use indexmap::IndexMap;
use owo_colors::{ OwoColorize};
use futures::{stream, StreamExt};
use tokio::sync::Mutex;
use indicatif::ProgressBar;
use rand::seq::IndexedRandom;
use crate::utils;

pub async fn select_import_options(root_path: &Path, server_url: &Url) {
    let post_url = server_url.join("characters/new").unwrap();
    
    // Let's search for the _characters folder
    let characters_path = root_path.join("src/_characters");

    if !characters_path.is_dir() {
        println!("{}", "Can't find src/_characters folder within the given path!");
        return;
    }

    let all_character_paths: Vec<PathBuf> = fs::read_dir(characters_path)
                .unwrap() // TODO: Instead of panicking, give user explanation of what happened.
                .filter_map(|file| file.ok())
                .map(|file| file.path())
                .filter(|path| !path.file_name().unwrap().to_string_lossy().starts_with("_"))
                .collect();    

    let total_character_amount = all_character_paths.len();

    println!("Character folder found! There are {} characters. {}", &total_character_amount, "Any files starting with _ were ignored.".italic());

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
                if let Err(import_errs) = import_given_characters(&all_character_paths, &post_url).await {
                    println!("---{}---\n{}\n------", "There were errors during the import!".red(), import_errs.join("\n"))
                }

                break;
            }

            "2" => { 
                let amount_of_characters = {
                    println!("How many characters would you like?");

                    loop {
                        let chosen_amount = crate::read_line().unwrap();

                        if let Ok(parsed_amount) =  chosen_amount.trim().parse::<usize>() {
                            match parsed_amount {
                                x if x < 1 => println!("{}", "That's too little!".yellow()),
                                x if x > total_character_amount => {
                                    println!("{}", "That's too much! Clamping down to ".yellow());
                                    break total_character_amount;
                                }
                                x => {
                                    break x;
                                }
                            }
                        }
                        else {
                            println!("{}", "I didn't quite get that.".yellow());
                        }
                    }
                };

                let random_characters = all_character_paths
                        .choose_multiple(&mut rand::rng(), amount_of_characters)
                        .map(|x| x.to_path_buf()).collect();
                if let Err(import_errs) = import_given_characters(&random_characters, &post_url).await {
                    println!("---{}---\n{}\n------", "There were errors during the import!".red(), import_errs.join("\n"))
                }
                break;
            }

            "3" => { 
                println!("What file would you like to import?");
                loop {
                    let chosen_file = crate::read_line().unwrap();

                    let trimmed_file = chosen_file.trim();

                    let chosen_file = all_character_paths.iter().find(|path| path.file_name().unwrap_or_default().eq_ignore_ascii_case(trimmed_file));

                    if let Some(file_path) = chosen_file {
                        if let Err(import_errs) = import_given_characters(&vec![file_path.to_owned()], &post_url).await {
                            println!("---{}---\n{}\n------", "There were errors during the import!".red(), import_errs.join("\n"));
                        }
                        break;
                    }
                    else {
                        println!("{}", "I didn't quite get that.".yellow());
                    }
                }
                break;
            }

            "0" => {
                break;
            }
            _ => println!("{}", "I didn't quite get that.".yellow())
        }
    }
}

async fn import_given_characters(character_file_paths: &Vec<PathBuf>, server_url: &Url) -> Result<Vec<String>, Vec<String>> {
    let simultaneous_threads = 4;
    let import_errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let import_successes: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // TODO: If try_into doesn't work, use a spinner.
    let progress_bar = Arc::new(ProgressBar::new(character_file_paths.len().try_into().unwrap()).with_finish(indicatif::ProgressFinish::AndClear));

    stream::iter(character_file_paths)
                    .for_each_concurrent(simultaneous_threads, |character_path| {
                        let import_errors_clone = import_errors.clone();
                        let import_successes_clone = import_successes.clone();
                        let progress_bar_clone = progress_bar.clone();

                        async move {
                            match import_given_character(character_path, server_url).await {
                                Err(mut import_error) => {  
                                    import_error.insert_str(0, &format!("{:?} ", character_path.file_name().unwrap()));
                                    let mut import_errors_unlocked = import_errors_clone.lock().await;
                                    import_errors_unlocked.push(import_error);
                                },
                                Ok(import_success) => {
                                    let import_success_readable = format!("{:?} STATUS {}: {}", character_path.file_name().unwrap(), import_success.status(), import_success.text().await.unwrap_or_default());
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

async fn import_given_character(character_file_path: &Path, server_url: &Url) -> Result<Response, String> {
    // Read and parse file
    let file_contents = fs::read_to_string(character_file_path)
            .map_err(|err| format!("File Read Err: {}", err.to_string()))?;

    let parser = Matter::<YAML>::new();
    let parsed_file = parser.parse(&file_contents)
            .map_err(|err| format!("File Parse Err: {:?}", err))?;
    
    let frontmatter: CharacterFrontmatter = parsed_file.data.ok_or("File Parse Err: No Frontmatter")?;
    let file_content = parsed_file.content;
    let character_slug: String = character_file_path.file_name().unwrap().to_ascii_lowercase().to_str().unwrap().trim_end_matches(".md").to_owned().replace(" ", "-");

    // Set the required fields for the post request
    let mut post_request = reqwest::multipart::Form::new()
        .text("name", frontmatter.character_title)
        .text("creator", frontmatter.character_author)
        .text("slug", character_slug.clone())
        .text("relevant_tag", character_slug.clone())
        .text("thumbnail_url", format!("https://powerdown.wiki/assets/img/characters/thumbnails/{}.png", character_slug)) // TODO: Convert to file sending
        .text("page_img_url", format!("https://powerdown.wiki/assets/img/{}", frontmatter.character_img_file)) // TODO: Convert to file sending
        .text("subtitles", serde_json::to_string(&frontmatter.character_subtitle).map_err(|err| format!("Subtitle JSON Err: {}", err.to_string()))?)
        .text("infobox", serde_json::to_string(&frontmatter.infobox_data).map_err(|err| format!("Infobox JSON Err: {}", err.to_string()))?)
        .text("tag", character_slug.clone())
        ;

    if let Some(overlay_css) = frontmatter.overlay_css {
        post_request = post_request.text("overlay_css", overlay_css);
    }

    if !file_content.trim().is_empty() {
        post_request = post_request.text("page_contents", file_content);
    }

    if frontmatter.hide_character {
        post_request = post_request.text("is_hidden", "true");
    }

    if let Some(retirement_reason) = frontmatter.archival_reason {
        post_request = post_request.text("retirement_reason", retirement_reason);
    }

    if let Some(logo) = frontmatter.logo_file {
        post_request = post_request.text("logo", format!("https://powerdown.wiki/assets/img/characters/logos/{}", logo));
    }
    
    if frontmatter.is_main_character {
        post_request = post_request.text("is_main_character", "true");
    }

    if let Some(inpage_character_name) = frontmatter.inpage_character_title {
        post_request = post_request.text("long_name", inpage_character_name);
    }

    // Send the post request and hope for the best.
    return reqwest::Client::new().post(server_url.to_owned())
        .multipart(post_request).send()
        .await.map_err(|err| format!("Info Send Err: {}", err.to_string()));
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
    #[serde(rename = "character-subtitle", deserialize_with = "utils::string_or_vec")]
    character_subtitle: Vec<String>,
    #[serde(rename = "character-author")]
    character_author: String,
    #[serde(rename = "logo-file")]
    logo_file: Option<String>,
    #[serde(rename = "character-img-file")]
    character_img_file: String, // The way it's written is relative to /assets/img/. Account for that.
    birthday: Option<String>, // Written as MM-DD
    #[serde(rename = "infobox-data", deserialize_with = "utils::deserialize_string_map")]
    infobox_data: IndexMap<String, String>,
    // I'm dropping relationships, this feature sucks.
    #[serde(rename = "css-code")]
    overlay_css: Option<String>,

    #[serde(default)]
    #[serde(rename = "main-character")]
    is_main_character: bool, // If missing, assume false.

    // TODO: Handle ritual stuff
}

