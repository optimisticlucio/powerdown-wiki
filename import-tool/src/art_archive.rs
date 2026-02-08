use crate::utils::{self, PresignedUrlsResponse};
use gray_matter::{Matter, engine::YAML};
use owo_colors::OwoColorize;
use rand::seq::IndexedRandom;
use regex::Regex;
use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub async fn select_import_options(root_path: &Path, server_url: &Url) {
    let post_url = server_url.join("art/new").unwrap();

    // Let's search for the _art-archive folder
    let art_path = root_path.join("src/_art-archive");

    if !art_path.is_dir() {
        println!("Can't find src/_art-archive folder within the given path!");
        return;
    }

    let all_art_paths: Vec<PathBuf> = fs::read_dir(art_path)
        .unwrap() // TODO: Instead of panicking, give user explanation of what happened.
        .filter_map(|file| file.ok())
        .map(|file| file.path())
        .filter(|path| !path.file_name().unwrap().to_string_lossy().starts_with("_"))
        .collect();

    let total_file_amount = all_art_paths.len();

    println!(
        "Art Archive folder found! There are {} art pieces. {}",
        &total_file_amount,
        "Any files starting with _ were ignored.".italic()
    );

    println!(
        "Would you like to\n{}\n{}\n{}\nor {}?\n{}",
        "(1) Import all art".yellow(),
        "(2) Import a random group of art pieces".blue(),
        "(3) Import a specific file".green(),
        "(4) Import all arts whose filenames fit a regex".cyan(),
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
                    &all_art_paths,
                    &post_url,
                    &import_given_art_piece,
                )
                .await
                {
                    println!(
                        "---{}---\n{}\n------",
                        "There were errors during the import!".red(),
                        import_errs.join("\n")
                    )
                }

                break;
            }

            "2" => {
                let amount_of_art = {
                    println!("How many art pieces would you like?");

                    loop {
                        let chosen_amount = crate::read_line().unwrap();

                        if let Ok(parsed_amount) = chosen_amount.trim().parse::<usize>() {
                            match parsed_amount {
                                x if x < 1 => println!("{}", "That's too little!".yellow()),
                                x if x > total_file_amount => {
                                    println!("{}", "That's too much! Clamping down to ".yellow());
                                    break total_file_amount;
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

                let random_art = all_art_paths
                    .choose_multiple(&mut rand::rng(), amount_of_art)
                    .map(|x| x.to_path_buf())
                    .collect();
                if let Err(import_errs) = utils::run_multiple_imports(
                    root_path,
                    &random_art,
                    &post_url,
                    &import_given_art_piece,
                )
                .await
                {
                    println!(
                        "---{}---\n{}\n------",
                        "There were errors during the import!".red(),
                        import_errs.join("\n")
                    )
                }
                break;
            }

            "3" => {
                println!("What file would you like to import?");
                loop {
                    let chosen_file = crate::read_line().unwrap();

                    let trimmed_file = chosen_file.trim();

                    let chosen_file = all_art_paths.iter().find(|path| {
                        path.file_name()
                            .unwrap_or_default()
                            .eq_ignore_ascii_case(trimmed_file)
                    });

                    if let Some(file_path) = chosen_file {
                        if let Err(import_errs) = utils::run_multiple_imports(
                            root_path,
                            &vec![file_path.to_owned()],
                            &post_url,
                            &import_given_art_piece,
                        )
                        .await
                        {
                            println!(
                                "---{}---\n{}\n------",
                                "There were errors during the import!".red(),
                                import_errs.join("\n")
                            );
                        }
                        break;
                    } else {
                        println!("{}", "I didn't quite get that.".yellow());
                    }
                }
                break;
            }

            "4" => {
                println!("Please write the regex you'd like to search by.");
                let given_regex = crate::read_line().unwrap().trim().to_owned();

                let parsed_regex = match Regex::new(&given_regex) {
                    Ok(x) => x,
                    Err(err) => {
                        println!("Regex Parse Err: {err:?}");
                        continue;
                    }
                };

                let files_that_fit_regex: Vec<PathBuf> = all_art_paths
                    .iter()
                    .filter(|filename| {
                        parsed_regex.is_match(filename.file_name().unwrap().to_str().unwrap())
                    })
                    .map(|path_buf| path_buf.to_owned())
                    .collect();

                println!(
                    "There are {} files which fit this regex. Initiating operation.",
                    files_that_fit_regex.len()
                );

                if let Err(import_errs) = utils::run_multiple_imports(
                    root_path,
                    &files_that_fit_regex,
                    &post_url,
                    &import_given_art_piece,
                )
                .await
                {
                    println!(
                        "---{}---\n{}\n------",
                        "There were errors during the import!".red(),
                        import_errs.join("\n")
                    )
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

async fn import_given_art_piece(
    root_path: &Path,
    art_file_path: &Path,
    server_url: &Url,
) -> Result<Response, String> {
    // Read and parse file
    let file_contents = fs::read_to_string(art_file_path)
        .map_err(|err| format!("File Read Err: {err}"))?
        .lines()
        .map(|line| {
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
                                if (18..26).contains(&yy) {
                                    format!("date: 20{third}-{second}-{first}")
                                } else {
                                    format!("date: 20{first}-{second}-{third}")
                                }
                            } else {
                                converted
                            }
                        }
                        // Check if it's DD-MM-YYYY format
                        else if first.len() <= 2 && third.len() == 4 {
                            format!("date: {third}-{second}-{first}")
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
        })
        .collect::<Vec<_>>()
        .join("\n");

    let parser = Matter::<YAML>::new();
    let parsed_file = parser
        .parse(&file_contents)
        .map_err(|err| format!("File Parse Err: {err:?}, file is:-----\n{file_contents}\n----"))?;

    let frontmatter: ArtFrontmatter = parsed_file.data.ok_or("File Parse Err: No Frontmatter")?;
    let file_content = parsed_file.content;
    let art_slug: String = art_file_path
        .file_name()
        .unwrap()
        .to_ascii_lowercase()
        .to_str()
        .unwrap()
        .trim_end_matches(".md")
        .to_owned()
        .replace(" ", "-");

    let art_archive_folder_path: PathBuf = art_file_path
        .parent()
        .unwrap() // pd-archive/src/_characters
        .parent()
        .unwrap() // pd-archive/src
        .join("assets/img/art-archive"); // pd-archive/src/assets/img/art-archive

    let art_thumbnail_folder_path: PathBuf = art_archive_folder_path.join("thumbnails");

    let thumbnail_path_attempt = if let Some(listed_thumbnail_path) = &frontmatter.thumbnail_file {
        // Check if this path actually exists. If not, run the search.
        if art_thumbnail_folder_path
            .join(listed_thumbnail_path)
            .exists()
        {
            Some(listed_thumbnail_path.to_owned())
        } else {
            None
        }
    } else {
        None
    };

    let thumbnail_path: String = thumbnail_path_attempt.unwrap_or_else(|| {
        let file_name = frontmatter.img_files[0]
            .split(".")
            .next()
            .unwrap()
            .to_owned();

        // Assuming the thumbnail has the same name as the img:
        if art_thumbnail_folder_path.join(&file_name).exists() {
            return file_name;
        }

        if art_thumbnail_folder_path
            .join(format!("{}.png", &file_name))
            .exists()
        {
            return format!("{}.png", &file_name);
        }

        if art_thumbnail_folder_path
            .join(format!("{}.jpg", &file_name))
            .exists()
        {
            return format!("{}.jpg", &file_name);
        }

        // Well, fuck me.
        format!(
            "ERROR_{}_NOT_FOUND",
            art_thumbnail_folder_path
                .join(format!("{}.png", &file_name))
                .as_os_str()
                .to_str()
                .unwrap()
        )
        .to_owned()
    });

    let mut modified_tags = frontmatter.tags.clone();
    // This is for my own convenience to hunt for thumbnails I forgot to fill in. THIS TAG SHOULD NOT BE IN FINISHED SITE!!!
    if thumbnail_path.starts_with("ERROR") {
        modified_tags.push("thumbnail-miss".to_string());
    }

    let thumbnail_path = root_path
        .join("src/assets/img/art-archive/thumbnails")
        .join(&thumbnail_path);
    let thumbnail_img_bytes =
        fs::read(&thumbnail_path).map_err(|err| format!("THUMBNAIL READ ERR: {err}"))?;

    let presigned_url_request = reqwest::Client::new()
        .post(server_url.to_owned())
        .json(&utils::PostingSteps::<PostArt>::RequestPresignedURLs {
            file_amount: frontmatter.img_files.len() as u8 + 1,
        })
        .send()
        .await
        .map_err(|err| format!("Presigned Request Failed: {err}"))?;

    let mut presigned_url_response: PresignedUrlsResponse = presigned_url_request
        .json()
        .await
        .map_err(|err| format!("Response mapping failed: {err}"))?;

    // Upload thumbnail.
    let thumbnail_url = presigned_url_response.presigned_urls.pop().unwrap();
    utils::send_to_presigned_url(&thumbnail_url, thumbnail_img_bytes)
        .await
        .map_err(|err| format!("Thumbnail Upload Err: {err:?}"))?;

    for (index, target_url) in presigned_url_response.presigned_urls.iter().enumerate() {
        let img_relative_path = frontmatter.img_files.get(index).unwrap();

        let img_file_path = root_path
            .join("src/assets/img/art-archive")
            .join(img_relative_path.trim_start_matches("/"));

        let img_file_bytes = fs::read(&img_file_path).map_err(|err| {
            format!(
                "ERROR IN READING FILE WITH PATH {}, err: {}",
                &img_relative_path, err
            )
        })?;

        utils::send_to_presigned_url(target_url, img_file_bytes)
            .await
            .map_err(|err| format!("Img Upload Err: {err}"))?;
    }

    let post_art = PostArt {
        title: frontmatter.title,
        creators: frontmatter.artists,
        thumbnail_key: thumbnail_url,
        art_keys: presigned_url_response.presigned_urls.clone(),
        slug: art_slug,
        is_nsfw: frontmatter.tags.contains(&"nsfw".to_owned()),
        description: if file_content.is_empty() {
            None
        } else {
            Some(file_content)
        },
        tags: frontmatter
            .tags
            .into_iter()
            .filter(|tag| !["sfw", "nsfw"].contains(&tag.as_str()))
            .collect(),
        creation_date: frontmatter.date,
    };

    reqwest::Client::new()
        .post(server_url.to_owned())
        .json(&utils::PostingSteps::UploadMetadata(post_art))
        .send()
        .await
        .map_err(|err| format!("Art Post Push Failed: {err}"))
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

    date: chrono::NaiveDate,
}

fn default_format() -> Format {
    Format::Image
}

#[derive(Deserialize, Serialize)]
enum Format {
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "video")]
    Video,
}

#[derive(Serialize, Debug)]
struct PostArt {
    pub title: String,
    pub creators: Vec<String>,
    pub thumbnail_key: String,
    pub slug: String,
    pub is_nsfw: bool,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub art_keys: Vec<String>,
    pub creation_date: chrono::NaiveDate,
}
