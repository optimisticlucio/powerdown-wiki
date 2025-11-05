use axum::extract::multipart::Field;
use axum::extract::{Multipart, Path, State, Json};
use axum::response::{IntoResponse, Redirect};
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

/// A post request targeting a specific story will modify that story's content with the post request's.
pub async fn update_story(
        Path(story_slug): Path<String>,
        State(state): State<ServerState>, 
        mut multipart: Multipart,
    ) -> Result<impl IntoResponse, RootErrors> {
    let existing_story = PageStory::get_by_slug(&story_slug, state.db_pool.get().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)?)
        .await.ok_or(RootErrors::NOT_FOUND)?;

    let mut base_builder = BaseStoryBuilder::default(); // TODO: How do I insert the existing story into these
    let mut page_builder = PageStoryBuilder::default();

    while let Some(recieved_field) = multipart.next_field().await.map_err(|_| RootErrors::INTERNAL_SERVER_ERROR)? {
        insert_user_passed_value_into_builders(recieved_field, &mut page_builder, &mut base_builder).await?;
    }

    // TODO: Implement

    Ok(Redirect::to(&format!("/stories/", ))) 
}

async fn insert_user_passed_value_into_builders(recieved_field: Field<'_>, page_builder: &mut PageStoryBuilder, base_builder: &mut BaseStoryBuilder) -> Result<(), RootErrors> {
    let field_name = match recieved_field.name() {
        None => return Err(RootErrors::BAD_REQUEST("Recieved a field with no name.".to_owned())),
        Some(x) => x
    };

    match field_name {
        "slug" => {
            base_builder.slug(text_or_internal_err(recieved_field).await?);
        },
        "title" => {
            base_builder.title(text_or_internal_err(recieved_field).await?);
        },
        "tagline" => {
            page_builder.tagline(Some(text_or_internal_err(recieved_field).await?));
        },
        "description" => {
            base_builder.description(text_or_internal_err(recieved_field).await?);
        },
        "creators" => {
            // Expected format: "x,y,z".
            base_builder.creators(text_or_internal_err(recieved_field).await?.split(",").map(|x| x.trim().to_owned()).collect());
        },
        "creation_date" => {
            let sent_date = text_or_internal_err(recieved_field).await?;

            // Expects date in ISO 8601 format (YYYY-MM-DD)
            let date = chrono::NaiveDate::parse_from_str(&sent_date, "%F")
                .map_err(|_| RootErrors::BAD_REQUEST("Given invalid date. Please ensure your date was in the YYYY-MM-DD format.".to_owned()))?; 
            base_builder.creation_date(date);
        },
        "is_hidden" => {
            // If we recieved this at all, I assume it's true.
            base_builder.is_hidden(true);
        },
        "custom_css" => {
            page_builder.custom_css(Some(text_or_internal_err(recieved_field).await?));
        },
        // TODO: Prev and Next stories.
        "editors_note" => {
            page_builder.editors_note(Some(text_or_internal_err(recieved_field).await?));
        },
        "content" => {
            page_builder.content(text_or_internal_err(recieved_field).await?);
        },
        "inpage_title" => {
            page_builder.inpage_title(Some(text_or_internal_err(recieved_field).await?));
        }
        _ => return Err(RootErrors::BAD_REQUEST(format!("Invalid Field Recieved: {}", field_name)))
    }

    Ok(())
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