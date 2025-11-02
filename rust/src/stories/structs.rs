use derive_builder::Builder;
use postgres::Row;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};

#[derive(Clone, Builder, Serialize, Deserialize)]
pub struct BaseStory {
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