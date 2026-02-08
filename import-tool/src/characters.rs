use crate::utils::{self, PresignedUrlsResponse};
use chrono::NaiveDate;
use gray_matter::{Matter, engine::YAML};
use indexmap::IndexMap;
use owo_colors::OwoColorize;
use rand::seq::IndexedRandom;
use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub async fn select_import_options(root_path: &Path, server_url: &Url) {
    let post_url = server_url.join("characters/new").unwrap();

    // Let's search for the _characters folder
    let characters_path = root_path.join("src/_characters");

    if !characters_path.is_dir() {
        println!("Can't find src/_characters folder within the given path!");
        return;
    }

    let all_character_paths: Vec<PathBuf> = fs::read_dir(characters_path)
        .unwrap() // TODO: Instead of panicking, give user explanation of what happened.
        .filter_map(|file| file.ok())
        .map(|file| file.path())
        .filter(|path| !path.file_name().unwrap().to_string_lossy().starts_with("_"))
        .collect();

    let total_character_amount = all_character_paths.len();

    println!(
        "Character folder found! There are {} characters. {}",
        &total_character_amount,
        "Any files starting with _ were ignored.".italic()
    );

    println!(
        "Would you like to\n{}\n{}\nor {}?\n{}",
        "(1) Import all characters".yellow(),
        "(2) Import a random group of characters".blue(),
        "(3) Import a specific file".green(),
        "Press 0 to exit screen.".italic()
    );

    loop {
        let chosen_option = crate::read_line().unwrap();

        let trimmed_option = chosen_option.trim();

        match trimmed_option {
            "1" => {
                // Import all
                if let Err(import_errs) = utils::run_multiple_imports(
                    root_path,
                    &all_character_paths,
                    &post_url,
                    &import_given_character,
                )
                .await
                {
                    println!(
                        "---{}---\n{}\n------",
                        "There were errors during the import!".red(),
                        import_errs.join("\n")
                    )
                } else {
                    println!("---{}---", "Imports completed successfully!".green());
                }

                break;
            }

            "2" => {
                let amount_of_characters = {
                    println!("How many characters would you like?");

                    loop {
                        let chosen_amount = crate::read_line().unwrap();

                        if let Ok(parsed_amount) = chosen_amount.trim().parse::<usize>() {
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
                        } else {
                            println!("{}", "I didn't quite get that.".yellow());
                        }
                    }
                };

                let random_characters = all_character_paths
                    .choose_multiple(&mut rand::rng(), amount_of_characters)
                    .map(|x| x.to_path_buf())
                    .collect();

                if let Err(import_errs) = utils::run_multiple_imports(
                    root_path,
                    &random_characters,
                    &post_url,
                    &import_given_character,
                )
                .await
                {
                    println!(
                        "---{}---\n{}\n------",
                        "There were errors during the import!".red(),
                        import_errs.join("\n")
                    )
                } else {
                    println!("---{}---", "Imports completed successfully!".green());
                }
                break;
            }

            "3" => {
                println!("What file would you like to import?");
                loop {
                    let chosen_file = crate::read_line().unwrap();

                    let trimmed_file = chosen_file.trim();

                    let chosen_file = all_character_paths.iter().find(|path| {
                        path.file_name()
                            .unwrap_or_default()
                            .eq_ignore_ascii_case(trimmed_file)
                    });

                    if let Some(file_path) = chosen_file {
                        if let Err(import_errs) = utils::run_multiple_imports(
                            root_path,
                            &vec![file_path.to_owned()],
                            &post_url,
                            &import_given_character,
                        )
                        .await
                        {
                            println!(
                                "---{}---\n{}\n------",
                                "There were errors during the import!".red(),
                                import_errs.join("\n")
                            );
                        } else {
                            println!("---{}---", "Imports completed successfully!".green());
                        }
                        break;
                    } else {
                        println!("{}", "I didn't quite get that.".yellow());
                    }
                }
                break;
            }

            "0" => {
                break;
            }
            _ => println!("{}", "I didn't quite get that.".yellow()),
        }
    }
}

async fn import_given_character(
    root_path: &Path,
    character_file_path: &Path,
    server_url: &Url,
) -> Result<Response, String> {
    // Read and parse file
    let file_contents =
        fs::read_to_string(character_file_path).map_err(|err| format!("File Read Err: {err}"))?;

    let parser = Matter::<YAML>::new();
    let parsed_file = parser
        .parse(&file_contents)
        .map_err(|err| format!("File Parse Err: {err:?}"))?;

    let frontmatter: CharacterFrontmatter =
        parsed_file.data.ok_or("File Parse Err: No Frontmatter")?;
    let file_content = parsed_file.content;
    let character_slug: String = character_file_path
        .file_name()
        .unwrap()
        .to_ascii_lowercase()
        .to_str()
        .unwrap()
        .trim_end_matches(".md")
        .to_owned()
        .replace(" ", "-");

    let thumbnail_img_filename = format!("{}.png", character_slug.replace("-", " "));
    let thumbnail_img_bytes = fs::read(
        root_path
            .join("src/assets/img/characters/thumbnails")
            .join(&thumbnail_img_filename),
    )
    .map_err(|err| format!("THUMBNAIL READ ERR: {err}"))?;

    let page_img_bytes = fs::read(
        root_path
            .join("src/assets/img")
            .join(frontmatter.character_img_file.trim_start_matches("/")),
    )
    .map_err(|err| format!("PAGE IMG READ ERR: {err}"))?;

    let page_contents = file_content.trim();
    let page_contents = if page_contents.is_empty() {
        None
    } else {
        Some(page_contents.to_string())
    };

    // Ok we have all the info we need, let's request the presigned URLs for whatever we need.
    // It's the thumbnail, page image, and maybe a logo.
    let file_amount = 2 + if frontmatter.logo_file.is_some() {
        1
    } else {
        0
    };

    let presigned_url_request = reqwest::Client::new()
        .post(server_url.to_owned())
        .json(&utils::PostingSteps::<PageCharacter>::RequestPresignedURLs { file_amount })
        .send()
        .await
        .map_err(|err| format!("Presigned Request Failed: {err}"))?;

    let mut presigned_url_response: PresignedUrlsResponse = presigned_url_request
        .json()
        .await
        .map_err(|err| format!("Response mapping failed: {err:?}"))?;

    // Upload thumbnail.
    let thumbnail_key = presigned_url_response.presigned_urls.pop().unwrap();
    utils::send_to_presigned_url(&thumbnail_key, thumbnail_img_bytes)
        .await
        .map_err(|err| format!("Thumbnail Upload Err: {err}"))?;

    // Upload page art.
    let page_img_key = presigned_url_response.presigned_urls.pop().unwrap();
    utils::send_to_presigned_url(&page_img_key, page_img_bytes)
        .await
        .map_err(|err| format!("Thumbnail Upload Err: {err}"))?;

    // Upload logo
    let logo_url = match frontmatter.logo_file {
        Some(logo_path) => {
            let logo_folder = root_path.join("src/assets/img/characters/logos");
            let path_to_logo = logo_folder.join(&logo_path);
            let logo_bytes =
                fs::read(path_to_logo).map_err(|err| format!("LOGO READ ERR: {err}"))?;

            let logo_img_key = presigned_url_response.presigned_urls.pop().unwrap();
            utils::send_to_presigned_url(&logo_img_key, logo_bytes)
                .await
                .map_err(|err| format!("Logo Upload Err: {err}"))?;

            Some(logo_img_key)
        }
        None => None,
    };

    // Good, we're ready to send.
    let post_character = PageCharacter {
        name: frontmatter.character_title.clone(),
        creator: frontmatter.character_author,
        slug: character_slug.clone(),
        is_hidden: frontmatter.hide_character,
        tag: Some(character_slug.clone()),
        thumbnail_key,
        page_img_key,
        subtitles: frontmatter.character_subtitle,
        infobox: frontmatter
            .infobox_data
            .iter()
            .map(|(a, b)| InfoboxRow {
                title: a.clone(),
                description: b.clone(),
            })
            .collect(),
        is_main_character: frontmatter.is_main_character,
        birthday: frontmatter.birthday.map(|birthday_in_mm_dd| {
            NaiveDate::parse_from_str(&format!("2000-{birthday_in_mm_dd}"), "%Y-%m-%d").unwrap()
        }),
        overlay_css: frontmatter.overlay_css,
        custom_css: None,
        long_name: frontmatter.inpage_character_title,
        retirement_reason: frontmatter.archival_reason,
        logo_url,
        page_contents,
    };

    reqwest::Client::new()
        .post(server_url.to_owned())
        .json(&utils::PostingSteps::UploadMetadata(post_character))
        .send()
        .await
        .map_err(|err| format!("Character Post Push Failed: {err:?}"))
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
    #[serde(
        rename = "character-subtitle",
        deserialize_with = "utils::string_or_vec"
    )]
    character_subtitle: Vec<String>,
    #[serde(rename = "character-author")]
    character_author: String,
    #[serde(rename = "logo-file")]
    logo_file: Option<String>,
    #[serde(rename = "character-img-file")]
    character_img_file: String, // The way it's written is relative to /assets/img/. Account for that.
    birthday: Option<String>, // Written as MM-DD
    #[serde(
        rename = "infobox-data",
        deserialize_with = "utils::deserialize_string_map"
    )]
    infobox_data: IndexMap<String, String>,
    // I'm dropping relationships, this feature sucks.
    #[serde(rename = "css-code")]
    overlay_css: Option<String>,

    #[serde(default)]
    #[serde(rename = "main-character")]
    is_main_character: bool, // If missing, assume false.

                             // TODO: Handle ritual stuff
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageCharacter {
    pub is_hidden: bool,
    pub is_main_character: bool,
    pub slug: String,
    pub name: String,
    pub thumbnail_key: String,
    pub birthday: Option<chrono::NaiveDate>,
    pub long_name: Option<String>,
    pub subtitles: Vec<String>,
    pub creator: String,
    pub retirement_reason: Option<String>,
    pub tag: Option<String>,
    pub logo_url: Option<String>,
    pub page_img_key: String,
    pub infobox: Vec<InfoboxRow>,
    pub overlay_css: Option<String>,
    pub custom_css: Option<String>,
    pub page_contents: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InfoboxRow {
    pub title: String,
    pub description: String,
}
