use axum::Router;
use axum_extra::routing::RouterExt;
use axum::routing::{get, post};
use axum::{extract::{OriginalUri, Path, State}, response::IntoResponse};
use askama::Template;
use crate::stories::structs::BaseStory;
use crate::{ServerState, errs::RootErrors, user::User};
use crate::utils::template_to_response;
use http::Uri;

mod structs;
mod post;
mod page;

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(story_index))
            .route_with_tsr("/new", post(post::add_story))
            .route_with_tsr("/{story_slug}", get(page::story_page).post(post::update_story))
}

#[derive(Template)] 
#[template(path = "stories/index.html")]
struct StoryIndex<'a> {
        user: Option<User>,
        original_uri: Uri,

        stories: Vec<BaseStory>,
        
        current_page_number: i64,
        total_page_number: i64,

        first_page_url: Option<&'a str>,
        prev_page_url: Option<&'a str>,
        next_page_url: Option<&'a str>,
        last_page_url: Option<&'a str>,
}

pub async fn story_index(
    State(state): State<ServerState>,
    OriginalUri(original_uri): OriginalUri,
) -> impl IntoResponse {
    const AMOUNT_OF_STORIES_PER_PAGE: i64 = 12;

    let relevant_stories = BaseStory::get_art_from_index(state.db_pool.get().await.unwrap(), 0, AMOUNT_OF_STORIES_PER_PAGE).await;

    template_to_response(
        StoryIndex {
            user: None, // TODO: Connect this to user system.
            original_uri,

            stories: relevant_stories,

            current_page_number: 1,
            total_page_number: 1,

            first_page_url: None, // TODO
            prev_page_url: None, // TODO
            next_page_url: None, // TODO
            last_page_url: None, // TODO
        }
    )
}