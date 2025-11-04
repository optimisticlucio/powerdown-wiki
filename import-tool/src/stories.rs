use std::{fs, path::{Path, PathBuf}, sync::Arc};
use reqwest::{multipart, Response, Url};
use serde::{Deserialize, Serialize};
use gray_matter::{Matter, engine::YAML};
use owo_colors::{ OwoColorize};
use futures::{stream, StreamExt};
use tokio::sync::Mutex;
use indicatif::ProgressBar;
use rand::seq::IndexedRandom;
use crate::utils;

pub async fn select_import_options(root_path: &Path, server_url: &Url) {
    let post_url = server_url.join("stories/new").unwrap();
    
    // Let's search for the _art-archive folder
    let story_path = root_path.join("src/_stories");

    if !story_path.is_dir() {
        println!("{}", "Can't find src/_stories folder within the given path!");
        return;
    }

    let all_story_paths: Vec<PathBuf> = fs::read_dir(story_path)
                .unwrap() // TODO: Instead of panicking, give user explanation of what happened.
                .filter_map(|file| file.ok())
                .map(|file| file.path())
                .filter(|path| !path.file_name().unwrap().to_string_lossy().starts_with("_"))
                .collect();    

    let total_story_amount = all_story_paths.len();

    println!("Story folder found! There are {} stories. {}", &total_story_amount, "Any files starting with _ were ignored.".italic());

    println!("Would you like to\n{}\n{}\nor {}?\n{}", 
        "(1) Import all stories".yellow(), 
        "(2) Import a random group of stories".blue(), 
        "(3) Import a specific file".green(), 
        "Press 0 to exit screen.".italic());

    loop {
        let chosen_option = crate::read_line().unwrap();

        let trimmed_option = chosen_option.trim();

        match trimmed_option {
            "1" => { // Import all
                if let Err(import_errs) = utils::run_multiple_imports(&root_path, &all_story_paths, &post_url, &import_given_story).await {
                    println!("---{}---\n{}\n------", "There were errors during the import!".red(), import_errs.join("\n"))
                }
                else {
                    // TODO: Handle prev/sequel for stories!
                }

                break;
            }

            "2" => { 
                let amount_of_art = {
                    println!("How many art pieces would you like?");

                    loop {
                        let chosen_amount = crate::read_line().unwrap();

                        if let Ok(parsed_amount) =  chosen_amount.trim().parse::<usize>() {
                            match parsed_amount {
                                x if x < 1 => println!("{}", "That's too little!".yellow()),
                                x if x > total_story_amount => {
                                    println!("{}", "That's too much! Clamping down to ".yellow());
                                    break total_story_amount;
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

                let random_art = all_story_paths
                        .choose_multiple(&mut rand::rng(), amount_of_art)
                        .map(|x| x.to_path_buf()).collect();
                if let Err(import_errs) = utils::run_multiple_imports(&root_path, &random_art, &post_url, &import_given_story).await {
                    println!("---{}---\n{}\n------", "There were errors during the import!".red(), import_errs.join("\n"))
                }
                break;
            }

            "3" => { 
                println!("What file would you like to import?");
                loop {
                    let chosen_file = crate::read_line().unwrap();

                    let trimmed_file = chosen_file.trim();

                    let chosen_file = all_story_paths.iter().find(|path| path.file_name().unwrap_or_default().eq_ignore_ascii_case(trimmed_file));

                    if let Some(file_path) = chosen_file {
                        if let Err(import_errs) = utils::run_multiple_imports(&root_path, &vec![file_path.to_owned()], &post_url, &import_given_story).await {
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

async fn import_given_story(_: &Path, story_file_path: & Path, server_url: &Url) -> Result<Response, String> {
    // Read and parse file
    let file_contents = fs::read_to_string(story_file_path).map_err(|err| format!("File Read Err: {}", err.to_string()))?
                .lines().map(|line| {
                // Check if this line is a date field and convert dots to dashes
                if line.trim_start().starts_with("date:") {
                    let converted = line.replace('.', "-");
                    
                    // Check if format is DD-MM-YYYY or DD-MM-YY and convert to YYYY-MM-DD
                    if let Some((_, date_str)) = converted.split_once(':') {
                        let date_str = date_str.trim();
                        let parts: Vec<&str> = date_str.split('-').collect();
                        
                        if parts.len() == 3 {
                            let (first, second, third) = (parts[0], parts[1], parts[2]);
                            
                            // Check if it's DD-MM-YY format (YY between 18-26)
                            if first.len() <= 2 && third.len() == 2 {
                                if let Ok(yy) = third.parse::<u32>() {
                                    if yy >= 18 && yy <= 26 {
                                        format!("date: 20{}-{}-{}", third, second, first)
                                    } else {
                                        format!("date: 20{}-{}-{}", first, second, third)
                                    }
                                } else {
                                    converted
                                }
                            }
                            // Check if it's DD-MM-YYYY format
                            else if first.len() <= 2 && third.len() == 4 {
                                format!("date: {}-{}-{}", third, second, first)
                            } else {
                                converted
                            }
                        } else {
                            converted
                        }
                    } else {
                        converted
                    }
                } else {
                    line.to_string()
                }
                }).collect::<Vec<_>>().join("\n");

    let parser = Matter::<YAML>::new();
    let parsed_file = parser.parse(&file_contents)
            .map_err(|err| format!("File Parse Err: {:?}, file is:-----\n{}\n----", err, file_contents))?;
    
    let mut frontmatter: StoryFrontmatter = parsed_file.data.ok_or("File Parse Err: No Frontmatter")?;
    frontmatter.content = Some(parsed_file.content);

    // Send the post request and hope for the best.
    return reqwest::Client::new()
                .post(server_url.clone())
                .json(&frontmatter)
                .send().await.map_err(|err| format!("Info Send Err: {}", err.to_string()));
}

#[derive(Deserialize, Serialize)]
struct StoryFrontmatter {
    title: String,

    #[serde(rename(deserialize="in-page-title"))]
    inpage_title: Option<String>,

    tagline: Option<String>,

    #[serde(default = "default_description")]
    description: String,

    #[serde(rename(deserialize="author"), deserialize_with = "utils::string_or_vec")]
    creators: Vec<String>,
    
    #[serde(rename(serialize="creation_date"))]
    date: chrono::NaiveDate,

    tags: Vec<String>,

    #[serde(rename(deserialize="continuation-of"))]
    previous_story_slug: Option<String>,

    #[serde(rename(deserialize="sequel"))]
    next_story_slug: Option<String>,

    #[serde(default)]
    exclude_from_pagination: bool,

    #[serde(rename(deserialize="editors-note"))]
    editors_note: Option<String>,

    // Dropping audio readings because I used it exactly *once*

    #[serde(default)]
    content: Option<String>
}

fn default_description() -> String {
    "MISSING DESCRIPTION".to_string()
}