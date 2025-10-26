use axum::extract::{State, Multipart};
use axum::response::{IntoResponse, Redirect};
use crate::{ServerState, RootErrors};
use super::structs::{BaseStoryBuilder, PageStoryBuilder};
use crate::utils::text_or_internal_err;

pub async fn add_story(State(state): State<ServerState>, mut multipart: Multipart) -> Result<impl IntoResponse, RootErrors> {
    let mut base_story_builder = BaseStoryBuilder::default();
    let mut page_story_builder = PageStoryBuilder::default();

    while let Some(recieved_field) = multipart.next_field().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)? {
        let field_name = match recieved_field.name() {
            None => return Err(RootErrors::BAD_REQUEST("Recieved a field with no name.".to_owned())),
            Some(x) => x
        };

        match field_name {
            "slug" => {
                base_story_builder.slug(text_or_internal_err(recieved_field).await?);
            },
            "title" => {
                base_story_builder.title(text_or_internal_err(recieved_field).await?);
            },
            "tagline" => {
                page_story_builder.tagline(Some(text_or_internal_err(recieved_field).await?));
            },
            "description" => {
                base_story_builder.description(text_or_internal_err(recieved_field).await?);
            },
            "creators" => {
                // Expected format: "x,y,z".
                base_story_builder.creators(text_or_internal_err(recieved_field).await?.split(",").map(|x| x.trim().to_owned()).collect());
            },
            "creation_date" => {
                let sent_date = text_or_internal_err(recieved_field).await?;

                // Expects date in ISO 8601 format (YYYY-MM-DD)
                let date = chrono::NaiveDate::parse_from_str(&sent_date, "%F")
                    .map_err(|_| RootErrors::BAD_REQUEST("Given invalid date. Please ensure your date was in the YYYY-MM-DD format.".to_owned()))?; 
                base_story_builder.creation_date(date);
            },
            "is_hidden" => {
                // If we recieved this at all, I assume it's true.
                base_story_builder.is_hidden(true);
            },
            "custom_css" => {
                page_story_builder.custom_css(Some(text_or_internal_err(recieved_field).await?));
            },
            // TODO: Prev and Next stories.
            "editors_note" => {
                page_story_builder.editors_note(Some(text_or_internal_err(recieved_field).await?));
            },
            "content" => {
                page_story_builder.content(text_or_internal_err(recieved_field).await?);
            }
            _ => return Err(RootErrors::BAD_REQUEST(format!("Invalid Field Recieved: {}", field_name)))
        }
    }

    // All relevant data collected. Hopefully. Let's try and build.
    let base_story = base_story_builder.build().map_err(|err| RootErrors::BAD_REQUEST(err.to_string()))?;

    page_story_builder.base_story(base_story);
    let page_story = page_story_builder.build().map_err(|err| RootErrors::BAD_REQUEST(err.to_string()))?;

    // Now the story is ready to send to the DB.
    let db_connection = state.db_pool.get().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?;

    // Let's build our query.
    let mut columns: Vec<String> = Vec::new();
    let mut values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

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

    if let Some(prev_story) = &page_story.previous_story {
        columns.push("prev_story".to_string());
        values.push(&prev_story.id);
    }

    if let Some(next_story) = &page_story.next_story {
        columns.push("next_story".to_string());
        values.push(&next_story.id);
    }

    if let Some(editors_note) = &page_story.editors_note {
        columns.push("editors_note".to_string());
        values.push(editors_note);
    }


    let query = format!("INSERT INTO story ({}) VALUES ({});",
            columns.join(","),
            columns.iter().enumerate().map(|(i, _)| format!("${}",i+1)).collect::<Vec<String>>().join(","),
        );

    db_connection.execute(&query, &values).await.map_err(|err| {
        println!("[STORY] Error in db query execution!\nQuery: {}\nError: {:?}", query, err);
        RootErrors::INTERNAL_SERVER_ERROR
    })?;

    Ok(Redirect::to(&format!("/stories/{}", page_story.base_story.slug))) 
}