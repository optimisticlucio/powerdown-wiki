use std::path::PathBuf;

use import_tool::{select_import_type, select_main_folder, select_server_url};
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() {
    println!(
"{}
This tool should be used regularly in testing and only {} on the final site.

{}",
"Hello! Welcome to the Power Down Wiki's Import Tool!".blue(),
"once".red().bold().underline(), 
"If there's any bugs, message Lucio over discord with screenshots.".yellow().italic());
    
    //let path_to_root = select_main_folder(); TODO: Uncomment if given to someone else
    let path_to_root = PathBuf::from("E:/Documents/Coding/unbridled-confidence/pd-archive"); 
    
    let server_url = select_server_url().await; // TODO: Uncomment once we go live!
    //let server_url = Url::parse("http://localhost:8080").unwrap();

    select_import_type(&path_to_root, &server_url).await;
}
