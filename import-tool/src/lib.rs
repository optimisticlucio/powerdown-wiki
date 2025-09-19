use std::io;
use owo_colors::OwoColorize;

mod art_archive;
mod characters;

pub fn read_line() -> Result<String, io::Error>{
    let mut input_string = String::new();

    io::stdin()
        .read_line(&mut input_string)?;

    return Ok(input_string);
}


/// Selects what to import, and then runs the relevant piece of code.
pub fn select_import_type() -> () {
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
            "0" => {
                break;
            }
            _ => println!("{}", "I didn't quite get that.".yellow())
        }
    }
}