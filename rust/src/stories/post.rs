use axum::extract::multipart::Field;
use axum::extract::{Json, Multipart, OriginalUri, Path, State};
use axum::response::{IntoResponse, Redirect};
use http::Uri;
use crate::{ServerState, RootErrors};
use super::structs::{BaseStoryBuilder, PageStoryBuilder, PageStory};
use crate::utils::text_or_internal_err;

pub async fn add_story(State(state): State<ServerState>, Json(recieved_story): Json<PageStory>) -> Result<impl IntoResponse, RootErrors> {
    let db_connection = state.db_pool.get().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

    // Let's build our query.
    let (columns, values) = set_columns_and_values_for_sql_query(&recieved_story, Vec::new(), Vec::new()).await;

    let query = format!("INSERT INTO story ({}) VALUES ({});",
            columns.join(","),
            columns.iter().enumerate().map(|(i, _)| format!("${}",i+1)).collect::<Vec<String>>().join(","),
        );

    db_connection.execute(&query, &values).await.map_err(|err| {
        println!("[STORY] Error in db query execution!\nQuery: {}\nError: {:?}", query, err);
        RootErrors::INTERNAL_SERVER_ERROR
    })?;

    Ok(Redirect::to(&format!("/stories/{}", recieved_story.base_story.slug))) 
}

async fn set_columns_and_values_for_sql_query<'a>
        (page_story: &'a PageStory, 
        mut columns: Vec<String>, 
        mut values: Vec<&'a (dyn tokio_postgres::types::ToSql + Sync)>)
        -> 
        (Vec<String>, Vec<&'a (dyn tokio_postgres::types::ToSql + Sync)>)
    {
    columns.push("page_slug".to_owned());
    values.push(&page_story.base_story.slug);

    columns.push("title".to_string());
    values.push(&page_story.base_story.title);

    columns.push("description".to_string());
    values.push(&page_story.base_story.description);

    columns.push("creators".to_string());
    values.push(&page_story.base_story.creators);

    columns.push("creation_date".to_string());
    values.push(&page_story.base_story.creation_date);

    // TODO: SANITIZE
    columns.push("content".to_string());
    values.push(&page_story.content);

    if page_story.base_story.is_hidden {
        columns.push("is_hidden".to_string());
        values.push(&true);
    }

    if let Some(tagline) = &page_story.tagline {
        columns.push("tagline".to_string());
        values.push(tagline);
    }

    if let Some(custom_css) = &page_story.custom_css {
        // TODO: SANITIZE
        columns.push("custom_css".to_string());
        values.push(custom_css);
    }

    if let Some(prev_story) = &page_story.previous_story_slug {
        // TODO: CHECK IF SLUG EXISTS, IF NOT, POINT TO NOTHING. IF DOES, GET ID.
    }

    if let Some(next_story) = &page_story.next_story_slug {
        // TODO: CHECK IF SLUG EXISTS, IF NOT, POINT TO NOTHING. IF DOES, GET ID.
    }

    if let Some(editors_note) = &page_story.editors_note {
        columns.push("editors_note".to_string());
        values.push(editors_note);
    }

    if let Some(inpage_title) = &page_story.inpage_title {
        columns.push("inpage_title".to_string());
        values.push(inpage_title);
    }

    (columns, values) // I return these instead of setting &mut because tokio_postgres expects immutable arrays.
}