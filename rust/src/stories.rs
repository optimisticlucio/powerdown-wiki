use crate::stories::structs::{BaseStory, StorySearchParameters};
use crate::utils::template_to_response;
use crate::RootErrors;
use crate::{user::User, ServerState};
use askama::Template;
use axum::extract::Query;
use axum::extract::{OriginalUri, State};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use axum_extra::routing::RouterExt;
use http::Uri;
use std::cmp::{self, min};

mod page;
mod post;
mod structs;

pub fn router() -> Router<ServerState> {
    Router::new()
        .route("/", get(story_index))
        .route_with_tsr("/new", post(post::add_story))
        .route_with_tsr("/{story_slug}", get(page::story_page))
}

#[derive(Debug, Template)]
#[template(path = "stories/index.html")]
struct StoryIndex {
    user: Option<User>,
    original_uri: Uri,

    stories: Vec<BaseStory>,

    current_page_number: i64,
    total_page_number: i64,

    first_page_url: Option<String>,
    prev_page_url: Option<String>,
    next_page_url: Option<String>,
    last_page_url: Option<String>,
}

pub async fn story_index(
    State(state): State<ServerState>,
    Query(search_params): Query<StorySearchParameters>,
    OriginalUri(original_uri): OriginalUri,
    cookie_jar: tower_cookies::Cookies,
) -> Result<Response, RootErrors> {
    const AMOUNT_OF_STORIES_PER_PAGE: i64 = 12;

    let total_story_amount =
        BaseStory::get_total_amount(state.db_pool.get().await.unwrap(), &search_params)
            .await
            .unwrap();

    // Total / per_page, rounded up.
    let total_page_number =
        (total_story_amount + AMOUNT_OF_STORIES_PER_PAGE - 1) / AMOUNT_OF_STORIES_PER_PAGE;

    // The requested page, with a minimal value of 1 and maximal value of the total pages available.
    let page_number_to_show = cmp::max(1, min(total_page_number, search_params.page));

    let relevant_stories = BaseStory::get_from_index(
        state.db_pool.get().await.unwrap(),
        (page_number_to_show - 1) * AMOUNT_OF_STORIES_PER_PAGE,
        AMOUNT_OF_STORIES_PER_PAGE,
        &search_params
    )
    .await;

    Ok(template_to_response(StoryIndex {
        user: User::easy_get_from_cookie_jar(&state, &cookie_jar).await?,
        original_uri,

        stories: relevant_stories,

        current_page_number: page_number_to_show,
        total_page_number,

        first_page_url: if page_number_to_show <= 2 {
            None
        } else {
            Some(get_search_url(StorySearchParameters {
                page: 1,
                ..search_params.clone()
            }))
        },
        prev_page_url: if page_number_to_show == 1 {
            None
        } else {
            Some(get_search_url(StorySearchParameters {
                page: page_number_to_show - 1,
                ..search_params.clone()
            }))
        },
        next_page_url: if page_number_to_show >= total_page_number {
            None
        } else {
            Some(get_search_url(StorySearchParameters {
                page: page_number_to_show + 1,
                ..search_params.clone()
            }))
        },
        last_page_url: if page_number_to_show >= total_page_number - 1 {
            None
        } else {
            Some(get_search_url(StorySearchParameters {
                page: total_page_number,
                ..search_params.clone()
            }))
        },
    }))
}

/// Given relevant query parameters, returns the relative URL of that story search.
fn get_search_url(params: StorySearchParameters) -> String {
    format!("/stories{}", params.to_uri_parameters(true))
}
