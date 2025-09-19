use import_tool::read_line;
use owo_colors::OwoColorize;

fn main() {
    println!(
"{}
This tool should be used regularly in testing and only {} on the final site.

{}",
"Hello! Welcome to the Power Down Wiki's Import Tool!".blue(),
"once".red().bold().underline(), 
"If there's any bugs, message Lucio over discord with screenshots.".yellow().italic());
    
    println!(
"\nWhat would you like to import?
(1) Characters.
(2) Art.
(3) Stories.
(9) Everything!
(0) Close tool"
    );
    loop {
        let chosen_option = read_line().unwrap();

        let trimmed_option = chosen_option.trim();

        match trimmed_option {
            "0" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("{}", "I didn't quite get that.".yellow())
        }
    }
}
