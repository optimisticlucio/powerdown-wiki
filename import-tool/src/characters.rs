use std::{fs, path::Path};
use reqwest::{Url};
use serde::{Deserialize, Serialize};
use serde::de::{Deserializer, Error};
use gray_matter::{Matter, engine::YAML};
use indexmap::IndexMap;

pub async fn select_import_options(root_path: &Path, server_url: &Url) {
    unimplemented!()
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