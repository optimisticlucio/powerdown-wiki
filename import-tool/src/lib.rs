use std::io;

mod art_archive;
mod characters;

pub fn read_line() -> Result<String, io::Error>{
    let mut input_string = String::new();

    io::stdin()
        .read_line(&mut input_string)?;

    return Ok(input_string);
}