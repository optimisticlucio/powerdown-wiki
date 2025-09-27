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
    
    let path_to_root = select_main_folder();

    let server_url = select_server_url();

    select_import_type(&path_to_root, &server_url).await;
}
