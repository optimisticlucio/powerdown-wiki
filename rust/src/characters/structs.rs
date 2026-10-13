use chrono;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use postgres::Row;
use postgres_types::{FromSql, ToSql};
use serde::Deserialize;

use crate::user::{User, UsermadePost};

#[derive(Debug, Clone, Deserialize)]
/// Info relevant to absolute most uses of a character
pub struct BaseCharacter {
    #[serde(skip)]
    pub db_id: i32, // The internal ID. Shouldn't be shown to user.
    #[serde(default)]
    pub is_hidden: bool,
    #[serde(skip)] // This should be learned by reference to other values
    pub is_archived: bool,
    #[serde(default)]
    pub is_main_character: bool,
    pub slug: String,
    pub name: String,
    pub thumbnail_key: String,
    #[serde(default)]
    pub birthday: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
/// Info relevant only to the page usage of a character
pub struct PageCharacter {
    #[serde(flatten)]
    pub base_character: BaseCharacter,

    #[serde(default)]
    pub long_name: Option<String>,
    pub subtitles: Vec<String>,
    pub creator: String,
    #[serde(default)]
    pub retirement_reason: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,

    #[serde(default)]
    pub logo_url: Option<String>,
    pub page_img_key: String,

    pub infobox: Vec<InfoboxRow>,
    #[serde(default)]
    pub overlay_css: Option<String>,
    #[serde(default)]
    pub custom_css: Option<String>,
    #[serde(default)]
    pub page_contents: Option<String>,
}

#[derive(Debug, FromSql, ToSql, Clone, Deserialize, PartialEq)]
#[postgres(name = "infobox_row")]
pub struct InfoboxRow {
    pub title: String,
    pub description: String,
}

impl BaseCharacter {
    /// Returns the page info of a single character, found by their page slug. If no such character exists, returns None.
    pub async fn get_by_slug(db_connection: &Object<Manager>, slug: &String) -> Option<Self> {
        let character_row = db_connection
            .query_one("SELECT * FROM character WHERE page_slug=$1", &[&slug])
            .await
            .ok()?;

        Some(Self::try_from(&character_row).unwrap())
    }

    /// Gets BaseCharacter for all characters in the database.
    pub async fn get_all_characters(db_connection: &Object<Manager>) -> Vec<Self> {
        // TODO: Select only what's necessary to speed it up.
        let character_rows = db_connection
            .query(
                "SELECT * FROM character WHERE post_state='public' ORDER BY short_name",
                &[],
            )
            .await
            .unwrap();

        character_rows
            .iter()
            .map(|row| Self::try_from(row).unwrap())
            .collect()
    }

    /// Gets only the characters who's birthday is today. The date is enforced by the Postgres DB.
    pub async fn get_birthday_characters(db_connection: &Object<Manager>) -> Vec<Self> {
        // TODO: Select only what's necessary to speed it up.
        let character_rows = db_connection.query(
            "SELECT * FROM character WHERE post_state='public' AND retirement_reason IS NULL AND EXTRACT(MONTH FROM birthday) = EXTRACT(MONTH FROM CURRENT_DATE) AND EXTRACT(DAY FROM birthday) = EXTRACT(DAY FROM CURRENT_DATE) ORDER BY short_name",
            &[]).await.unwrap();

        character_rows
            .iter()
            .map(|row| Self::try_from(row).unwrap())
            .collect()
    }

    /// Gets a random character from the database.
    pub async fn get_random_character(db_connection: &Object<Manager>) -> Self {
        let character_row = db_connection
            .query_one(
                "SELECT * FROM character WHERE post_state='public' ORDER BY RANDOM() LIMIT 1",
                &[],
            )
            .await
            .unwrap();

        Self::try_from(&character_row).unwrap()
    }
}

impl TryFrom<&Row> for BaseCharacter {
    type Error = postgres::Error;
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let archival_reason: Option<String> = row.try_get("retirement_reason")?;

        Ok(Self {
            db_id: row.try_get("id")?,
            is_hidden: row.try_get("is_hidden")?,
            is_main_character: row.try_get("is_main_character")?,
            is_archived: archival_reason.is_some(),
            name: row.try_get("short_name")?,
            thumbnail_key: row.try_get("thumbnail")?,
            slug: row.try_get("page_slug")?,
            birthday: row.try_get("birthday")?,
        })
    }
}

impl PartialEq for BaseCharacter {
    fn eq(&self, other: &Self) -> bool {
        (self.db_id == other.db_id) && (self.slug == other.slug)
    }
}

impl PartialOrd for BaseCharacter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for BaseCharacter {}

impl Ord for BaseCharacter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PageCharacter {
    /// Returns the page info of a single character, found by their page slug. If no such character exists, returns None.
    pub async fn get_by_slug(db_connection: &Object<Manager>, slug: &str) -> Option<PageCharacter> {
        let character_row = db_connection
            .query_one("SELECT * FROM character WHERE page_slug=$1", &[&slug])
            .await
            .ok()?;

        Some(Self::try_from(&character_row).unwrap())
    }
}

impl TryFrom<&Row> for PageCharacter {
    type Error = postgres::Error;
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            base_character: BaseCharacter::try_from(row)?,
            long_name: row.try_get("long_name")?,
            subtitles: row.try_get("subtitles")?,
            creator: row.try_get("creator")?,
            retirement_reason: row.try_get("retirement_reason")?,
            logo_url: row.try_get("logo")?,
            page_img_key: row.try_get("page_image")?,
            infobox: row.try_get("infobox")?,
            overlay_css: row.try_get("overlay_css")?,
            custom_css: row.try_get("custom_css")?,
            page_contents: row.try_get("page_text")?,
            tag: row.try_get("relevant_tag")?,
        })
    }
}

impl PartialEq for PageCharacter {
    fn eq(&self, other: &Self) -> bool {
        self.base_character.eq(&other.base_character)
    }
}

impl PartialOrd for PageCharacter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PageCharacter {}

impl Ord for PageCharacter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.base_character.cmp(&other.base_character)
    }
}

impl UsermadePost for PageCharacter {
    fn can_be_modified_by(&self, user: &User) -> bool {
        user.user_type.permissions().can_modify_others_content
            || user
                .creator_name
                .as_ref()
                .is_some_and(|creator_name| &self.creator == creator_name)
    }
}
