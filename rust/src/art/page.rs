use axum::{extract::{Path, State}, response::{ IntoResponse}};
use askama::Template;
use crate::{errs::RootErrors, user::User, ServerState, utils::template_to_response};
use super::structs;

#[derive(Template)] 
#[template(path = "art/page.html")]
struct ArtPage {
    user: Option<User>,

    title: String,
    artists: Vec<String>,
    formatted_creation_date: String,
    art_urls: Vec<String>,
    tags: Vec<String>,
    description: Option<String>, // Assumed to be markdown.
}

impl ArtPage {
    /// Given a URL, returns true if it's one that should be wrapped in a <video> tag.
    fn url_is_of_video(&self, url: &&String) -> bool {
        ["mov", "mp4", "avi"].iter().any(|ext| url.ends_with(ext))
    }
}

pub async fn character_page(
    Path(art_slug): Path<String>,
    State(state): State<ServerState>
) -> impl IntoResponse {
    if let Some(requested_art) = structs::PageArt::get_by_slug(state.db_pool.get().await.unwrap(), art_slug).await {
        template_to_response(
            ArtPage {
                user: None, // TODO: Connect this to user system.

                title: requested_art.base_art.title,
                artists: requested_art.base_art.creators,
                formatted_creation_date: requested_art.creation_date.to_string(),
                art_urls: requested_art.art_urls,
                tags: requested_art.tags,
                description: requested_art.description
            }
        )
    }   
    else {
        RootErrors::NOT_FOUND.into_response()
    }
}