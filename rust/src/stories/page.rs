use axum::{extract::{OriginalUri, Path, State}, response::IntoResponse};
use askama::Template;
use http::Uri;
use rand::seq::IndexedRandom;
use crate::{ServerState, errs::RootErrors, stories::structs::{self, BaseStory}, user::User};
use crate::utils::template_to_response;
use comrak::{ markdown_to_html};



#[derive(Template)] 
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

        content: &'a str,
}

pub async fn art_page(
    Path(story_slug): Path<String>,
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
) -> impl IntoResponse {
    if let Some(requested_story) = structs::PageStory::get_by_slug(&story_slug, state.db_pool.get().await.unwrap()).await {
        template_to_response(
            StoryPage {
                user: None, // TODO: Connect this to user system.
                original_uri,

                story_title: &requested_story.inpage_title.unwrap_or(requested_story.base_story.title),
                tagline: requested_story.tagline.as_deref(),
                authors: &requested_story.base_story.creators,

                editors_note: requested_story.editors_note.as_deref(),
                next_story: None, // TODO: Map next_story_slug to the story
                prev_story: None, // TODO: Map prev_story_slug to the story

                content: &requested_story.content
            }
        )
    }   
    else {
        RootErrors::NOT_FOUND.into_response()
    }
}