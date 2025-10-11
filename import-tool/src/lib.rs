use std::{io, path::{Path, PathBuf}};
use owo_colors::OwoColorize;
use reqwest::Url;

mod art_archive;
mod characters;
mod stories;
pub mod utils;

pub fn read_line() -> Result<String, io::Error>{
    let mut input_string = String::new();

    io::stdin()
        .read_line(&mut input_string)?;

    return Ok(input_string);
}

pub fn select_main_folder() -> PathBuf {
    println!("Please input the path of the {} folder inside of unbridled-confidence:", 
    "pd-archive".bold());

    loop {
        let chosen_option = read_line().unwrap();

        let trimmed_option = chosen_option.trim();

        // TODO: Ensure that it's a valid path, and that it does match the structure of pd-archive. If not, ask for another path.

        return PathBuf::from(trimmed_option);
    }
}


pub fn select_server_url() -> Url {
    println!("Please input the URL of the server we're targeting:");

    loop {
        let chosen_option = read_line().unwrap();

        let trimmed_option = chosen_option.trim();

        // TODO: Ensure that it's a valid URL and that it matches what we're looking for.

        return Url::parse(trimmed_option).unwrap(); // TODO: Handle parse error
    }
}

/// Selects what to import, and then runs the relevant piece of code.
pub async fn select_import_type(root_path: &Path, server_url: &Url) -> () {
    println!(
"\nWhat would you like to import?
(1) Characters.
(2) Art.
(3) Stories.
(9) Everything!
(0) Nothing else, I'm good."
    );
    loop {
        let chosen_option = read_line().unwrap();

        let trimmed_option = chosen_option.trim();

        match trimmed_option {
            "1" => {
                characters::select_import_options(root_path, server_url).await;
                break;
            }

            "2" => {
                art_archive::select_import_options(root_path, server_url).await;
                break;
            }

            "3" => {
                stories::select_import_options(root_path, server_url);
                break;
            }

            "9" => {
                unimplemented!("Haven't implemented full import yet. You don't need it yet, calm down.");
            }

            "0" => {
                break;
            }
            _ => println!("{}", "I didn't quite get that.".yellow())
        }
    }
}