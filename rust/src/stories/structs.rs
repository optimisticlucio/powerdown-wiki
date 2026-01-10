use derive_builder::Builder;
use postgres::Row;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};

#[derive(Clone, Builder, Serialize, Deserialize)]
pub struct BaseStory {
    #[serde(default)]
    pub id: i32,
    pub title: String,
    pub description: String,
    pub creators: Vec<String>,
    pub creation_date: chrono::NaiveDate,
    pub slug: String,
    #[builder(default = false)]
    pub is_hidden: bool
}

#[derive(Clone, Builder, Serialize, Deserialize)]
pub struct PageStory {
    #[serde(flatten)]
    pub base_story: BaseStory,
    #[builder(default = None)]
    pub inpage_title: Option<String>,
    #[builder(default = None)]
    pub tagline: Option<String>,
    pub tags: Vec<String>,
    #[builder(default = None)]
    pub previous_story_slug: Option<String>,
    #[builder(default = None)]
    pub next_story_slug: Option<String>,
    #[builder(default = None)]
    pub custom_css: Option<String>,
    #[builder(default = None)]
    pub editors_note: Option<String>,

    pub content: String,
}

impl BaseStory {
    /// Gets [amount_to_return] amount of stories, starting from the [index] newest one.
    pub async fn get_art_from_index(db_connection: Object<Manager>, index: i64, amount_to_return: i64) -> Vec<Self>{
        // TODO: Narrow down select so it runs faster.
        let query_parameters: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&amount_to_return, &index];

        let requested_story_rows = db_connection.query(
            "SELECT * FROM story ORDER BY creation_date DESC LIMIT $1 OFFSET $2",
            &query_parameters).await.unwrap();

        requested_story_rows.iter().map(Self::from_db_row).collect()
    }

    /// Returns the base info of a single story, found by their page slug. If no such story exists, returns None.
    pub async fn get_by_slug(slug: String, db_connection: Object<Manager>) -> Option<Self> {
        // TODO: Limit search
        let story_row = db_connection.query_one(
            "SELECT * FROM story WHERE page_slug=$1",
            &[&slug]).await.ok()?;

        Some(Self::from_db_row(&story_row))
    }

    /// Converts a DB row with the relevant info to a BaseStory struct.
    pub fn from_db_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            creators: row.get("creators"),
            creation_date: row.get("creation_date"),
            slug: row.get("page_slug"),
            is_hidden: row.get("is_hidden")
        }
    }

    /// Gets [amount_to_return] amount of stories, starting from the [index] newest piece.
    pub async fn get_from_index(db_connection: Object<Manager>, index: i64, amount_to_return: i64, search_parameters: &StorySearchParameters) -> Vec<Self>{
        // TODO: Narrow down select so it runs faster.
        let mut query_parameters: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&amount_to_return, &index];

        let query_where = search_parameters.get_postgres_where(&mut query_parameters);

        let query = format!("SELECT * FROM story {} ORDER BY creation_date DESC LIMIT $1 OFFSET $2", query_where);

        let requested_rows = db_connection.query(
            &query,
            &query_parameters).await.unwrap();

        requested_rows.iter().map(Self::from_db_row).collect()
    }

    /// Returns the total amount of stories currently in the db.
    pub async fn get_total_amount(db_connection: Object<Manager>, search_params: &StorySearchParameters) -> Result<i64, Box<dyn std::error::Error>> {
        let mut query_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();

        let query_where = search_params.get_postgres_where(&mut query_params);

        let query = format!("SELECT COUNT(page_slug) FROM story {}", query_where);

        let row = db_connection
            .query_one(&query, &query_params)
            .await?;

        let count: i64 = row.get(0);
        Ok(count)
    }
}

impl PageStory {
    /// Returns the page info of a single story, found by their page slug. If no such story exists, returns None.
    pub async fn get_by_slug(slug: &str, db_connection: Object<Manager>) -> Option<Self> {
        let story_row = db_connection.query_one(
            "SELECT * FROM story WHERE page_slug=$1",
            &[&slug]).await.ok()?;

        Some(Self::from_db_row(&story_row))
    }

    /// Converts a DB row with the relevant info to a PageStory struct.
    pub fn from_db_row(row: &Row) -> Self {
        Self {
            base_story: BaseStory::from_db_row(row),
            inpage_title: row.get("inpage_title"),
            tagline: row.get("tagline"),
            tags: row.get("tags"),
            previous_story_slug: None, // TODO
            next_story_slug: None, // TODO
            custom_css: row.get("custom_css"),
            editors_note: row.get("editors_note"),
            content: row.get("content")
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct StorySearchParameters {
    #[serde(default = "default_page_number")]
    pub page: i64,

    #[serde(default)]
    pub tags: Vec<String>,
}
fn default_page_number() -> i64 {
    1
}

impl StorySearchParameters {
    /// Returns how the parameter section of a URL with these parameters should look like.
    pub fn to_uri_parameters(&self, include_page_number: bool) -> String {
        let mut parameters: Vec<String> = Vec::new();

        if include_page_number && self.page > 1 {
            parameters.push(format!("page={}", self.page));
        }

        // -- Return --

        if parameters.is_empty() {
            "".to_string()
        }
        else {
            format!("?{}", parameters.join("&"))
        }
    }

    /// Creates the WHERE section of a postgresql statement for these parameters. Modifies a given set of function parameters.
    /// Lifetime of parameter modifications tied to lifetime of struct.
    pub fn get_postgres_where<'a>(&'a self, params: &mut Vec<&'a (dyn tokio_postgres::types::ToSql + Sync)>) -> String{
        let mut query_conditions: Vec<String> = Vec::new();

        query_conditions.push("NOT is_hidden".to_string());

        if !self.tags.is_empty() {
            params.push(&self.tags);
            query_conditions.push(format!("tags @> ${}", params.len()));
        }

        // --- Return ---
        if query_conditions.is_empty() {
            String::new()
        }
        else {
            format!("WHERE {}", query_conditions.join(" AND "))
        }
    }
}
