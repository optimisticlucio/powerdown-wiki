use import_tool::{select_import_type};
use owo_colors::OwoColorize;

fn main() {
    println!(
"{}
This tool should be used regularly in testing and only {} on the final site.

{}",
"Hello! Welcome to the Power Down Wiki's Import Tool!".blue(),
"once".red().bold().underline(), 
"If there's any bugs, message Lucio over discord with screenshots.".yellow().italic());
    
    // TODO: Get the location of where the files are.

    select_import_type();
}
