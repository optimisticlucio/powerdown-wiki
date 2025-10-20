use std::{fs, path::{Path, PathBuf}, sync::Arc};
use reqwest::{multipart, Response, Url};
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Error as DeError};
use gray_matter::{Matter, engine::YAML};
use indexmap::IndexMap;
use owo_colors::{ OwoColorize};
use futures::{stream, StreamExt};
use tokio::sync::Mutex;
use indicatif::ProgressBar;
use rand::seq::IndexedRandom;
use crate::utils;

pub async fn select_import_options(root_path: &Path, server_url: &Url) {
    let post_url = server_url.join("art/new").unwrap();
    
    // Let's search for the _art-archive folder
    let art_path = root_path.join("src/_art-archive");

    if !art_path.is_dir() {
        println!("{}", "Can't find src/_art-archive folder within the given path!");
        return;
    }

    let all_art_paths: Vec<PathBuf> = fs::read_dir(art_path)
                .unwrap() // TODO: Instead of panicking, give user explanation of what happened.
                .filter_map(|file| file.ok())
                .map(|file| file.path())
                .filter(|path| !path.file_name().unwrap().to_string_lossy().starts_with("_"))
                .collect();    

    let total_art_amount = all_art_paths.len();

    println!("Art Archive folder found! There are {} art pieces. {}", &total_art_amount, "Any files starting with _ were ignored.".italic());

    println!("Would you like to\n{}\n{}\nor {}?\n{}", 
        "(1) Import all art".yellow(), 
        "(2) Import a random group of art pieces".blue(), 
        "(3) Import a specific file".green(), 
        "Press 0 to exit screen.".italic());

    loop {
        let chosen_option = crate::read_line().unwrap();

        let trimmed_option = chosen_option.trim();

        match trimmed_option {
            "1" => { // Import all
                if let Err(import_errs) = import_given_art(&root_path, &all_art_paths, &post_url).await {
                    println!("---{}---\n{}\n------", "There were errors during the import!".red(), import_errs.join("\n"))
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
                                x if x > total_art_amount => {
                                    println!("{}", "That's too much! Clamping down to ".yellow());
                                    break total_art_amount;
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

                let random_art = all_art_paths
                        .choose_multiple(&mut rand::rng(), amount_of_art)
                        .map(|x| x.to_path_buf()).collect();
                if let Err(import_errs) = import_given_art(&root_path, &random_art, &post_url).await {
                    println!("---{}---\n{}\n------", "There were errors during the import!".red(), import_errs.join("\n"))
                }
                break;
            }

            "3" => { 
                println!("What file would you like to import?");
                loop {
                    let chosen_file = crate::read_line().unwrap();

                    let trimmed_file = chosen_file.trim();

                    let chosen_file = all_art_paths.iter().find(|path| path.file_name().unwrap_or_default().eq_ignore_ascii_case(trimmed_file));

                    if let Some(file_path) = chosen_file {
                        if let Err(import_errs) = import_given_art(&root_path, &vec![file_path.to_owned()], &post_url).await {
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

async fn import_given_art(root_path: &Path, art_file_paths: &Vec<PathBuf>, server_url: &Url) -> Result<Vec<String>, Vec<String>> {
    let simultaneous_threads = 4;
    let import_errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let import_successes: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // TODO: If try_into doesn't work, use a spinner.
    let progress_bar = Arc::new(ProgressBar::new(art_file_paths.len().try_into().unwrap()).with_finish(indicatif::ProgressFinish::AndClear));

    stream::iter(art_file_paths)
                    .for_each_concurrent(simultaneous_threads, |art_path| {
                        let import_errors_clone = import_errors.clone();
                        let import_successes_clone = import_successes.clone();
                        let progress_bar_clone = progress_bar.clone();

                        async move {
                            match import_given_art_piece(&root_path, art_path, server_url).await {
                                Err(mut import_error) => {  
                                    import_error.insert_str(0, &format!("{:?} ", art_path.file_name().unwrap()));
                                    let mut import_errors_unlocked = import_errors_clone.lock().await;
                                    import_errors_unlocked.push(import_error);
                                },
                                Ok(import_success) => {
                                    let import_success_readable = format!("{:?} STATUS {}: {}", art_path.file_name().unwrap(), import_success.status(), import_success.text().await.unwrap_or_default());
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

async fn import_given_art_piece(root_path: &Path, art_file_path: &Path, server_url: &Url) -> Result<Response, String> {
    // Read and parse file
    let file_contents = fs::read_to_string(art_file_path).map_err(|err| format!("File Read Err: {}", err.to_string()))?
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
    
    let frontmatter: ArtFrontmatter = parsed_file.data.ok_or("File Parse Err: No Frontmatter")?;
    let file_content = parsed_file.content;
    let art_slug: String = art_file_path.file_name().unwrap().to_ascii_lowercase().to_str().unwrap().trim_end_matches(".md").to_owned().replace(" ", "-");

    let art_archive_folder_path: PathBuf = art_file_path.parent().unwrap() // pd-archive/src/_characters
                                    .parent().unwrap() // pd-archive/src
                                    .join("assets/img/art-archive"); // pd-archive/src/assets/img/art-archive

    let art_thumbnail_folder_path: PathBuf = art_archive_folder_path.join("thumbnails");

    let thumbnail_path_attempt = if let Some(listed_thumbnail_path) = &frontmatter.thumbnail_file {
        // Check if this path actually exists. If not, run the search.
        if art_thumbnail_folder_path.join(listed_thumbnail_path).exists() {
            Some(listed_thumbnail_path.to_owned())
        }
        else {
            None
        }
    } else { None };

    let thumbnail_path: String = thumbnail_path_attempt.unwrap_or_else(|| {
            let file_name = frontmatter.img_files[0].split(".").next().unwrap().to_owned();

            // Assuming the thumbnail has the same name as the img:
            if art_thumbnail_folder_path.join(&file_name).exists() {
                return file_name;
            }

            if art_thumbnail_folder_path.join(format!("{}.png", &file_name)).exists() {
                return format!("{}.png", &file_name);
            } 

            if art_thumbnail_folder_path.join(format!("{}.jpg", &file_name)).exists() {
                return format!("{}.jpg", &file_name);
            } 

            // Well, fuck me.
            format!("ERROR_{}_NOT_FOUND", art_thumbnail_folder_path.join(format!("{}.png", &file_name)).as_os_str().to_str().unwrap()).to_owned()
        });

    let mut modified_tags = frontmatter.tags.clone();
    // This is for my own convenience to hunt for thumbnails I forgot to fill in. THIS TAG SHOULD NOT BE IN FINISHED SITE!!!
    if thumbnail_path.starts_with("ERROR") {
        modified_tags.push("thumbnail-miss".to_string()); 
    }

    let thumbnail_path = root_path.join("src/assets/img/art-archive/thumbnails").join(&thumbnail_path);
    let thumbnail_img_bytes = fs::read(&thumbnail_path)
            .map_err(|err| format!("THUMBNAIL READ ERR: {}", err.to_string()))?;
    let thumbnail_filename = thumbnail_path.file_name().ok_or("THUMBNAIL DOES NOT HAVE FILENAME".to_owned())?.to_str().unwrap().to_string();

    // Set the required fields for the post request
    let mut post_request = reqwest::multipart::Form::new()
        .text("slug", art_slug)
        .text("creation_date", frontmatter.date.format("%F").to_string())
        .text("title", frontmatter.title)
        .text("creators", serde_json::to_string(&frontmatter.artists).map_err(|err| format!("Artist JSON Err: {}", err.to_string()))?)
        .part("thumbnail", multipart::Part::bytes(thumbnail_img_bytes).file_name(thumbnail_filename)) 
        .text("tags", serde_json::to_string(&modified_tags.iter().filter(|tag| !["nsfw".to_owned(), "sfw".to_owned()].contains(tag)).collect::<Vec<&String>>())
                .map_err(|err| format!("Tag JSON Err: {}", err.to_string()))?)
        ;

    for (index, img_file_relative_path) in frontmatter.img_files.iter().enumerate() {
        let img_file_path = root_path.join("src/assets/img/art-archive").join(img_file_relative_path.trim_start_matches("/"));

        let img_file_bytes = fs::read(&img_file_path).map_err(|err| format!("ERROR IN READING FILE WITH PATH {}, err: {}", &img_file_relative_path, err.to_string()))?;

        let img_file_filename = img_file_path.file_name().ok_or(format!("FILE {} DOES NOT HAVE FILENAME", img_file_relative_path))?.to_str().unwrap().to_string();

        post_request = post_request.part(format!("file_{}", index), multipart::Part::bytes(img_file_bytes).file_name(img_file_filename));
    }

    if frontmatter.tags.contains(&"nsfw".to_owned()) {
        post_request = post_request.text("is_nsfw", "true");
    }

    if !file_content.is_empty() {
        post_request = post_request.text("description", file_content);
    }

    // Send the post request and hope for the best.
    return reqwest::Client::new().post(server_url.to_owned())
        .multipart(post_request).send()
        .await.map_err(|err| format!("Info Send Err: {}", err.to_string()));
}

#[derive(Deserialize, Serialize)]
struct ArtFrontmatter {
    title: String,

    #[serde(default = "default_format")]
    format: Format,

    #[serde(rename = "img-file", deserialize_with = "utils::string_or_vec")]
    img_files: Vec<String>,

    #[serde(rename = "thumbnail-file")]
    thumbnail_file: Option<String>, // If it's None, we NEED to search for the actual thumbnail. It must be Some by the submission time!

    #[serde(rename = "artist", deserialize_with = "utils::string_or_vec")]
    artists: Vec<String>,

    tags: Vec<String>,

    date: chrono::NaiveDate
}

fn default_format() -> Format {
    Format::Image
}

#[derive(Deserialize, Serialize)]
enum Format {
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "video")]
    Video
}