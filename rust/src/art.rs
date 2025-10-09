use askama::Template;
use axum::{response::Html, routing::get, routing::post, extract::State, Router};
use crate::{errs::RootErrors, ServerState};
use deadpool::managed::Object;
use deadpool_postgres::Manager;

mod page;
mod search;
mod structs;
mod post;

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(art_index))
        .route("/new", post(post::add_character))
}

#[derive(Template)] 
#[template(path = "art/index.html")]
struct ArtIndexPage {
    random_quote: String,

    current_page_number: u32,
    total_page_number: u32,

    first_page_url: Option<String>,
    prev_page_url: Option<String>,
    next_page_url: Option<String>,
    last_page_url: Option<String>,

    art_pieces: Vec<structs::BaseArt>
}

async fn art_index(State(state): State<ServerState>) -> Html<String> {
    let amount_of_art_per_page: u32 = 24;

    let random_quote = {
        let statement = "SELECT * FROM quote WHERE association = 'quote' ORDER BY RANDOM() LIMIT 1;"; 

        match state.db_pool.get().await {
            // TODO: Turn this unwrap into something that handles error better.
            Ok(db_connection) => 
                db_connection.query(statement, &[]).await.unwrap()
                    .get(0).unwrap()
                    .get(0),
            _ => "Insert funny text here.".to_owned() // "Oh shit it broke" text that won't seem too odd for a random user.
        }
    };

    let art_pieces = structs::BaseArt::get_art_from_index(state.db_pool.get().await.unwrap(), 0, amount_of_art_per_page).await;

    ArtIndexPage {
        random_quote,

        current_page_number: 1,
        total_page_number: get_total_amount_of_art(state.db_pool.get().await.unwrap()).await.unwrap() / amount_of_art_per_page,

        first_page_url: None, // TODO
        prev_page_url: None, // TODO
        next_page_url: None, // TODO
        last_page_url: None, // TODO

        art_pieces
    }.render().unwrap().into()
}

/// Returns the total amount of art currently in the db.
pub async fn get_total_amount_of_art(db_connection: Object<Manager>) -> Result<u32, Box<dyn std::error::Error>> {
    let row = db_connection
        .query_one("SELECT COUNT(page_slug) FROM art", &[])
        .await?;
    
    let count: u32 = row.get(0);
    Ok(count)
}