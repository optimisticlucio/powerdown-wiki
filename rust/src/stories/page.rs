use axum::{extract::{OriginalUri, Path, State}, response::{IntoResponse, Response}};
use askama::Template;
use http::Uri;
use rand::seq::IndexedRandom;
use crate::{ServerState, errs::RootErrors, stories::structs::{self, BaseStory}, user::User};
use crate::utils::template_to_response;
use comrak::{ markdown_to_html};
use ammonia::clean;

#[derive(Debug, Template)]
#[template(path = "stories/page.html")]
struct StoryPage<'a> {
        user: Option<User>,
        original_uri: Uri,

        story_title: &'a str,
        tagline: Option<&'a str>,
        authors: &'a Vec<String>,
        editors_note: Option<&'a str>,

        prev_story: Option<BaseStory>,
        next_story: Option<BaseStory>,

        custom_css: Option<&'a str>,

        content: &'a str,
}

pub async fn story_page(
    Path(story_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    if let Some(requested_story) = structs::PageStory::get_by_slug(&story_slug, state.db_pool.get().await.unwrap()).await {

        // TODO: Sanitize custom_css.
        // TODO: Properly clean style. Clean both using the same ammonia settings!

        let converted_story = {
            let mut parsing_options = comrak::Options::default();
            parsing_options.render.unsafe_ = true; // Allow HTML in input.

            let unsafe_story = markdown_to_html(&requested_story.content, &parsing_options);

            // Sanitize output.
            let mut ammonia_settings = ammonia::Builder::default();
            ammonia_settings.add_generic_attributes(&["style", "class"]); // TODO: Properly clean Style. It's an attack vector!

            ammonia_settings.clean(&unsafe_story).to_string()
        };

        Ok(template_to_response(
            StoryPage {
                user: User::easy_get_from_cookie_jar(&state, &cookie_jar).await?,
                original_uri,

                story_title: &requested_story.inpage_title.unwrap_or(requested_story.base_story.title),
                tagline: requested_story.tagline.as_deref(),
                authors: &requested_story.base_story.creators,

                editors_note: requested_story.editors_note.as_deref(),
                next_story: None, // TODO: Map next_story_slug to the story
                prev_story: None, // TODO: Map prev_story_slug to the story

                custom_css: requested_story.custom_css.as_deref(),

                content: &converted_story
            }
        ))
    }
    else {
        Err(RootErrors::NOT_FOUND(original_uri, cookie_jar))
    }
}
