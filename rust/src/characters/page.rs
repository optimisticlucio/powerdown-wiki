use crate::utils::template_to_response;
use crate::{
    characters::{structs::InfoboxRow, PageCharacter},
    errs::RootErrors,
    user::User,
    ServerState,
};
use askama::Template;
use axum::{
    extract::{OriginalUri, Path, State},
    response::Response,
};
use comrak::markdown_to_html;
use http::Uri;
use rand::seq::IndexedRandom;

#[derive(Debug, Template)]
#[template(path = "characters/page.html")]
struct CharacterPage<'a> {
    user: Option<User>,
    original_uri: Uri,

    retirement_reason: Option<&'a str>,
    overlay_css: Option<&'a str>,
    custom_css: Option<&'a str>,

    creator: &'a str,
    tag: Option<&'a str>,

    name: &'a str,
    subtitle: &'a str,
    infobox: Vec<InfoboxRow>,

    page_img: &'a str,
    character_logo: Option<&'a str>,

    content: Option<&'a str>,
}

pub async fn character_page(
    Path(character_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    if let Some(chosen_char) =
        PageCharacter::get_by_slug(character_slug, state.db_pool.get().await.unwrap()).await
    {
        let parsed_content = chosen_char.page_contents.map(|contents| {
            parse_character_page_contents(&contents).unwrap_or("PARSING FAILED!".to_owned())
        });
        let retirement_reason = chosen_char
            .retirement_reason
            .as_ref()
            .map(|f| markdown_to_html(&f, &comrak::Options::default()));

        Ok(template_to_response(CharacterPage {
            user: User::easy_get_from_cookie_jar(&state, &cookie_jar).await?,
            original_uri,

            retirement_reason: retirement_reason.as_deref(),
            overlay_css: chosen_char.overlay_css.as_deref(),
            custom_css: chosen_char.custom_css.as_deref(),
            creator: &chosen_char.creator,
            tag: chosen_char.tag.as_deref(),

            name: chosen_char
                .long_name
                .clone()
                .unwrap_or(chosen_char.base_character.name.clone())
                .as_ref(),

            subtitle: chosen_char.subtitles.choose(&mut rand::rng()).unwrap(),
            infobox: chosen_char.infobox.clone(),
            page_img: &chosen_char.page_img_url,
            character_logo: chosen_char.logo_url.as_deref(),
            content: parsed_content.as_deref(),
        }))
    } else {
        Err(RootErrors::NotFound(original_uri, cookie_jar))
    }
}

fn parse_character_page_contents(unparsed_contents: &str) -> Option<String> {
    let mut parsed_contents = markdown_to_html(
        unparsed_contents,
        &comrak::Options {
            extension: comrak::ExtensionOptions {
                ..comrak::ExtensionOptions::default()
            },
            parse: comrak::ParseOptions {
                ..comrak::ParseOptions::default()
            },
            render: comrak::RenderOptions {
                ..comrak::RenderOptions::default()
            },
        },
    )
    .replace("<h1>", r#"</div> <h1> <span>"#)
    .replace("</h1>", r#"</span> </h1> <div class="text">"#);

    if let Some(first_added_div_exit_tag) = parsed_contents.find("</div>") {
        let text_exists_before_first_header = first_added_div_exit_tag > 0;

        if text_exists_before_first_header {
            parsed_contents.insert_str(0, r#"<div class="text">"#);
        } else {
            parsed_contents = parsed_contents.replacen("</div>", "", 1);
        }
    }

    Some(parsed_contents)
}
