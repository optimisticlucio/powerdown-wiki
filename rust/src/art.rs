use std::cmp::{self, min};

use askama::Template;
use axum::{response::Html, routing::get, routing::post, extract::State, Router, extract::{Query}};
use crate::{errs::RootErrors, ServerState, user::User};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use structs::ArtSearchParameters;

mod page;
mod structs;
mod post;

pub fn router() -> Router<ServerState> {
    Router::new().route("/", get(art_index))
        .route("/new", post(post::add_art))
        .route("/new", get(post::art_posting_page))
        .route("/{art_slug}", get(page::character_page))
}

#[derive(Template)] 
#[template(path = "art/index.html")]
struct ArtIndexPage<'a> {
    user: Option<User>,

    random_quote: String,

    current_page_number: i64,
    total_page_number: i64,

    first_page_url: Option<String>,
    prev_page_url: Option<String>,
    next_page_url: Option<String>,
    last_page_url: Option<String>,

    art_pieces: Vec<structs::BaseArt>,

    all_tags: Vec<String>,

    user_search_params: &'a ArtSearchParameters
}

async fn art_index(
        State(state): State<ServerState>, 
        Query(query_params): Query<ArtSearchParameters>
    ) -> Html<String> {
    // Static Values
    let amount_of_art_per_page: i64 = 24;

    let random_quote = {
        let association = if query_params.nsfw { "sex_joke" } else { "quote" };

        let statement = format!("SELECT * FROM quote WHERE association = '{}' ORDER BY RANDOM() LIMIT 1;", association); 

        match state.db_pool.get().await {
            // TODO: Turn this unwrap into something that handles error better.
            Ok(db_connection) => 
                db_connection.query(&statement, &[]).await.unwrap()
                    .get(0).unwrap()
                    .get(0),
            _ => "Insert funny text here.".to_owned() // "Oh shit it broke" text that won't seem too odd for a random user.
        }
    };

    let total_amount_of_art = get_total_amount_of_art(state.db_pool.get().await.unwrap(), &query_params).await.unwrap();

    // Total / per_page, rounded up. 
    let total_pages_available_for_search =  (total_amount_of_art + amount_of_art_per_page - 1) / amount_of_art_per_page;

    // The requested page, with a minimal value of 1 and maximal value of the total pages available.
    let page_number_to_show = cmp::max(1, min(total_pages_available_for_search, query_params.page));

    let art_pieces = structs::BaseArt::get_art_from_index(
            state.db_pool.get().await.unwrap(), 
            (page_number_to_show - 1) * amount_of_art_per_page,
            amount_of_art_per_page.into(),
            &query_params
        ).await;

    ArtIndexPage {
        user: None, // TODO: Connect to user system.
        user_search_params: &query_params,

        random_quote,

        current_page_number: page_number_to_show,
        total_page_number: total_pages_available_for_search,

        first_page_url: if page_number_to_show <= 2 { None } else { 
            Some(get_search_url(ArtSearchParameters { page: 1, ..query_params.clone()}))
        }, 
        prev_page_url: if page_number_to_show == 1 { None } else { 
            Some(get_search_url(ArtSearchParameters { page: page_number_to_show - 1 , ..query_params.clone()}))
        }, 
        next_page_url: if page_number_to_show >= total_pages_available_for_search { None } else { 
            Some(get_search_url(ArtSearchParameters { page: page_number_to_show + 1 , ..query_params.clone()}))
        }, 
        last_page_url: if page_number_to_show >= total_pages_available_for_search - 1  { None } else { 
            Some(get_search_url(ArtSearchParameters { page: total_pages_available_for_search , ..query_params.clone()}))
        }, 

        all_tags: get_all_tags(state.db_pool.get().await.unwrap()).await,

        art_pieces
    }.render().unwrap().into()
}

/// Returns the total amount of art currently in the db. May be given tags to constrain the search
pub async fn get_total_amount_of_art(db_connection: Object<Manager>, search_params: &ArtSearchParameters) -> Result<i64, Box<dyn std::error::Error>> {
    let mut query_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

    let query_where = search_params.get_postgres_where(&mut query_params);

    let query = format!("SELECT COUNT(page_slug) FROM art {}", query_where);

    let row = db_connection
        .query_one(&query, &query_params)
        .await?;
    
    let count: i64 = row.get(0);
    Ok(count)
}

/// Given relevant query parameters, returns the relative URL of that art search.
fn get_search_url(params: ArtSearchParameters) -> String {
    format!("/art{}", params.to_uri_parameters(true))
}

/// Returns all the unique tags in all art.
// TODO: Should probably cache this. Not a frequently changing field, and even if it does, a short discrepancy is ok.
pub async fn get_all_tags(db_connection: Object<Manager>) -> Vec<String> {
    let answers = db_connection
        .query("SELECT DISTINCT unnest(tags) AS tags FROM art;", &[]).await
        .unwrap();

    answers.iter().map(|row| row.get(0)).collect::<Vec<String>>()
}