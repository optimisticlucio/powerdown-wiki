use std::io::Write;

use ammonia::url::form_urlencoded::parse;
use axum::{extract::Path, response::{ IntoResponse}};
use askama::Template;
use rand::seq::IndexedRandom;
use crate::{errs::RootErrors, navbar::Navbar};
use crate::test_data;
use crate::utils::template_to_response;
use comrak::{create_formatter, markdown_to_html};

#[derive(Template)] 
#[template(path = "characters/page.html")]
struct CharacterPage<'a> {
    navbar: Navbar,

    retirement_reason: Option<&'a str>,
    overlay_css: Option<&'a str>,

    creator: &'a str,
    tag: &'a str,
    
    name: &'a str,
    subtitle: &'a str,
    infobox: Vec<(String, String)>,

    page_img: &'a str,
    character_logo: Option<&'a str>,

    content: &'a str,
}

pub async fn character_page(
    Path(character_slug): Path<String>
) -> impl IntoResponse {
    // TODO: Actually connect to a database.

    if let Some(chosen_char) = test_data::get_test_characters().iter().find(|character| character.name.to_lowercase() == character_slug) {

        let parsed_content = parse_character_page_contents(&chosen_char.page_contents).unwrap_or("PARSING FAILED!".to_owned());
        let retirement_reason = chosen_char.archival_reason.as_ref().map(|f| markdown_to_html(&f, &comrak::Options::default()));

        template_to_response(
            CharacterPage {
                navbar: Navbar::not_logged_in(), //TODO: Hook up

                retirement_reason: retirement_reason.as_deref(),
                overlay_css: chosen_char.overlay_css.as_deref(), 
                creator: &chosen_char.author,
                tag: &chosen_char.name.to_ascii_lowercase().replace(" ", "-"),

                name: chosen_char.long_name.clone().unwrap_or(chosen_char.name.clone()).as_ref(),

                subtitle: chosen_char.subtitles.choose(&mut rand::rng()).unwrap(),
                infobox: chosen_char.infobox.clone(),
                page_img: &chosen_char.img_url,
                character_logo: chosen_char.logo_url.as_deref(),
                content: &parsed_content 
            }
        )
    }
    else {
        RootErrors::NOT_FOUND.into_response()
    }
}

fn parse_character_page_contents(unparsed_contents: &str) -> Option<String> {
    let mut parsed_contents = markdown_to_html(unparsed_contents, &comrak::Options::default())
        .replace("<h1>", r#"</div> <h1> <span>"#)
        .replace("</h1>", r#"</span> </h1> <div class="text">"#);

    if let Some(first_added_div_exit_tag) = parsed_contents.find("</div>") {
        let text_exists_before_first_header = first_added_div_exit_tag > 0;

        if text_exists_before_first_header {
            parsed_contents.insert_str(0, r#"<div class="text">"#);
        }
        else {
            parsed_contents = parsed_contents.replacen("</div>", "", 1);
        }
    }
    
    Some(parsed_contents)
}