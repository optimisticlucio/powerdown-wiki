use chrono;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use postgres::Row;
use postgres_types::{FromSql, ToSql, Type};
use derive_builder::Builder;

// TODO: Get character ritual info

#[derive(Clone, Builder)]
pub struct BaseCharacter { // Info relevant to absolute most uses of a character
    #[builder(default)]
    db_id: i32, // The internal ID. Shouldn't be shown to user.
    #[builder(default = false)]
    pub is_hidden: bool,
    #[builder(default = false)] // In some cases relies on a diff value.
    pub is_archived: bool,
    #[builder(default = false)]
    pub is_main_character: bool,
    pub slug: String,
    pub name: String,
    pub thumbnail_url: String,
}

#[derive(Clone, Builder)]
pub struct PageCharacter { // Info relevant to character page
    pub base_character: BaseCharacter,

    #[builder(default = None)]
    pub long_name: Option<String>,
    pub subtitles: Vec<String>,
    pub creator: String,
    #[builder(default = None)]
    pub archival_reason: Option<String>,
    #[builder(default = None)]
    pub tag: Option<String>, // TODO: Should this be optional?

    #[builder(default = None)]
    pub logo_url: Option<String>,
    pub page_img_url: String,

    pub infobox: Vec<InfoboxRow>,
    #[builder(default = None)]
    pub overlay_css: Option<String>,
    #[builder(default = None)]
    pub custom_css: Option<String>,
    pub page_contents: String
}

#[derive(Clone)]
pub struct RitualCharacter {
    pub base_character: BaseCharacter,

    pub power_name: String,
    pub power_description: String
}

#[derive(Debug, FromSql, ToSql, Clone)]
#[postgres(name = "infobox_row")]
pub struct InfoboxRow {
    pub title: String,
    pub description: String,
}

impl InfoboxRow {
    pub fn new(title: String, description: String) -> Self {
        Self {
            title,
            description
        }
    }
}

impl BaseCharacter {
    /// Gets BaseCharacter for all characters in the database.
    pub async fn get_all_characters(db_connection: Object<Manager>) -> Vec<BaseCharacter> {
        // TODO: Limit this query to only what's necessary to speed it up.
        let character_rows = db_connection.query(
            "SELECT * FROM character",
            &[]).await.unwrap();

        character_rows.iter().map(Self::from_db_row).collect()
    }

    /// Gets only the characters who's birthday is today. The date is enforced by the Postgres DB.
    pub async fn get_birthday_characters(db_connection: Object<Manager>) -> Vec<BaseCharacter> {
        // TODO: Limit this query to only what's necessary to speed it up.
        let character_rows = db_connection.query(
            "SELECT * FROM character WHERE EXTRACT(MONTH FROM birthday) = EXTRACT(MONTH FROM CURRENT_DATE) AND EXTRACT(DAY FROM birthday) = EXTRACT(DAY FROM CURRENT_DATE)",
            &[]).await.unwrap();

        character_rows.iter().map(Self::from_db_row).collect()
    }

    /// Converts a DB row with the relevant info to a BaseCharacter struct.
    fn from_db_row(row: &Row) -> Self {
        let archival_reason: Option<String> = row.get("retirement_reason");

        BaseCharacter {
            db_id: row.get("id"),
            is_hidden: row.get("is_hidden"),
            is_main_character: row.get("is_main_character"),
            is_archived: archival_reason.is_some(),
            name: row.get("short_name"),
            thumbnail_url: row.get("thumbnail"),
            slug: row.get("page_slug")
        }
    }
}

impl PageCharacter {
    /// Returns the page info of a single character, found by their page slug. If no such character exists, returns None.
    pub async fn get_by_slug(slug: String, db_connection: Object<Manager>) -> Option<PageCharacter> {
        let character_row = db_connection.query_one(
            "SELECT * FROM character WHERE page_slug=$1", 
            &[&slug]).await.ok()?;

        Some(Self::from_db_row(&character_row))
    }

    /// Converts a DB row with the relevant info to a PageCharacter struct.
    fn from_db_row(row: &Row) -> Self {
        let archival_reason: Option<String> = row.get("retirement_reason");

        Self {
            base_character: BaseCharacter {
                db_id: row.get("id"),
                is_hidden: row.get("is_hidden"),
                is_main_character: row.get("is_main_character"),
                is_archived: archival_reason.is_some(),
                name: row.get("short_name"),
                thumbnail_url: row.get("thumbnail"),
                slug: row.get("page_slug")
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
            page_contents: row.get("page_text"),
            tag: row.get("relevant_tag")
        }
    }
}

// TODO: impl RitualCharacter