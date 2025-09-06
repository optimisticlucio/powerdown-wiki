use chrono;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use postgres::Row;
use postgres_types::{FromSql, Type};

#[derive(Clone)]
pub struct Character { // TODO: Dump all places that use this.
    pub is_hidden: bool,
    pub archival_reason: Option<String>, // If none, not archived.

    pub name: String,
    pub long_name: Option<String>,
    pub subtitles: Vec<String>,
    pub author: String,
    pub logo_url: Option<String>,
    // TODO: character birthday
    pub thumbnail_url: String,
    pub img_url: String,
    pub infobox: Vec<(String, String)>,
    // TODO: relationships?
    pub overlay_css: Option<String>,
    pub page_contents: String,
}

// TODO: Get character ritual info

#[derive(Clone)]
pub struct BaseCharacter { // Info relevant to absolute most uses of a character
    db_id: i64, // The internal ID. Shouldn't be shown to user.
    pub is_hidden: bool,
    pub is_archived: bool,
    pub name: String,
    pub thumbnail_url: String,
}

#[derive(Clone)]
pub struct PageCharacter { // Info relevant to character page
    pub base_character: BaseCharacter,

    pub long_name: Option<String>,
    pub subtitles: Vec<String>,
    pub creator: String,
    pub archival_reason: Option<String>,

    pub logo_url: Option<String>,
    pub page_img_url: String,

    pub infobox: Vec<InfoboxRow>,
    pub overlay_css: Option<String>,
    pub custom_css: Option<String>,
    pub page_contents: String
}

#[derive(Debug, FromSql, Clone)]
struct InfoboxRow {
    title: String,
    description: String,
}

impl BaseCharacter {
    pub async fn get_all_characters(db_connection: Object<Manager>) -> Vec<BaseCharacter> {
        // TODO: Limit this query to only what's necessary to speed it up.
        let character_rows = db_connection.query(
            "SELECT * FROM character",
            &[]).await.unwrap();

        character_rows.iter().map(Self::from_db_row).collect()
    }

    fn from_db_row(row: &Row) -> Self {
        let archival_reason: Option<String> = row.get("archival_reason");

        BaseCharacter {
            db_id: row.get("id"),
            is_hidden: row.get("is_hidden"),
            is_archived: archival_reason.is_some(),
            name: row.get("short_name"),
            thumbnail_url: row.get("thumbnail")
        }
    }
}

impl PageCharacter {
    /// Returns the page info of a single character, found by their page slug. If no such character exists, returns None.
    pub async fn get_by_slug(slug: String, db_connection: Object<Manager>) -> Option<PageCharacter> {
        let character_row = db_connection.query_one(
            "SELECT * FROM character WHERE page_slug='$1'", 
            &[&slug]).await.ok()?;

        Some(Self::from_db_row(&character_row))
    }

    fn from_db_row(row: &Row) -> Self {
        let archival_reason: Option<String> = row.get("archival_reason");

        Self {
            base_character: BaseCharacter {
                db_id: row.get("id"),
                is_hidden: row.get("is_hidden"),
                is_archived: archival_reason.is_some(),
                name: row.get("short_name"),
                thumbnail_url: row.get("thumbnail")
            },
            long_name: row.get("long_name"),
            subtitles: row.get("subtitles"),
            creator: row.get("creator"),
            archival_reason,
            logo_url: row.get("logo"),
            page_img_url: row.get("page_image"),
            infobox: row.get("infobox"),
            overlay_css: row.get("overlay_css"),
            custom_css: row.get("custom_css"),
            page_contents: row.get("page_text")
        }
    }
}