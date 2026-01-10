use chrono;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use postgres::Row;
use postgres_types::{FromSql, ToSql, Type};
use derive_builder::Builder;
use rand::{distr::Alphanumeric, Rng};

// TODO: Get character ritual info

#[derive(Debug, Clone, Builder)]
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
    #[builder(default = None)]
    pub birthday: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Builder)]
pub struct PageCharacter { // Info relevant to character page
    pub base_character: BaseCharacter,

    #[builder(default = None)]
    pub long_name: Option<String>,
    pub subtitles: Vec<String>,
    pub creator: String,
    #[builder(default = None)]
    pub retirement_reason: Option<String>,
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
    #[builder(default = None)]
    pub page_contents: Option<String>
}

#[derive(Debug, Clone)]
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
        // TODO: Select only what's necessary to speed it up.
        let character_rows = db_connection.query(
            "SELECT * FROM character ORDER BY short_name",
            &[]).await.unwrap();

        character_rows.iter().map(Self::from_db_row).collect()
    }

    /// Gets only the characters who's birthday is today. The date is enforced by the Postgres DB.
    pub async fn get_birthday_characters(db_connection: Object<Manager>) -> Vec<BaseCharacter> {
        // TODO: Select only what's necessary to speed it up.
        let character_rows = db_connection.query(
            "SELECT * FROM character WHERE EXTRACT(MONTH FROM birthday) = EXTRACT(MONTH FROM CURRENT_DATE) AND EXTRACT(DAY FROM birthday) = EXTRACT(DAY FROM CURRENT_DATE) ORDER BY short_name",
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
            slug: row.get("page_slug"),
            birthday: row.get("birthday"),
        }
    }

    /// Gets an unused ID in the DB, by creating a temp object in the DB and extracting its ID.
    /// WARNING: Remember to clean up the temp object if you end up not using the given ID.
    pub async fn get_unused_id(db_connection: Object<Manager>) -> i32 {
        let random_page_slug: String = rand::rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect();

        // There's a very slight chance this operation panics on correct behaviour
        // bc it uses random strings. It should probably be fine, but I should fix this someday.
        let insert_operation_result = db_connection.query_one(
            "INSERT INTO character (is_hidden, page_slug, short_name, subtitles, creator, thumbnail, page_image)
            VALUES (TRUE, $1, 'TEMP', ARRAY['Something you shouldn''t be seeing!'], 'RNJesus', '', '')
            RETURNING id", &[&random_page_slug]).await.unwrap();

        insert_operation_result.get(0) // id is int, which converts to i32.
    } 
}

impl PartialEq for BaseCharacter {
    fn eq(&self, other: &Self) -> bool {
        (self.db_id == other.db_id) && (self.slug == self.slug)
    }
}

impl PartialOrd for BaseCharacter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Eq for BaseCharacter {

}

impl Ord for BaseCharacter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
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
        let retirement_reason: Option<String> = row.get("retirement_reason");

        Self {
            base_character: BaseCharacter {
                db_id: row.get("id"),
                is_hidden: row.get("is_hidden"),
                is_main_character: row.get("is_main_character"),
                is_archived: retirement_reason.is_some(),
                name: row.get("short_name"),
                thumbnail_url: row.get("thumbnail"),
                slug: row.get("page_slug"),
                birthday: row.get("birthday"),
            },
            long_name: row.get("long_name"),
            subtitles: row.get("subtitles"),
            creator: row.get("creator"),
            retirement_reason,
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

impl PartialEq for PageCharacter {
    fn eq(&self, other: &Self) -> bool {
        self.base_character.eq(&other.base_character)
    }
}

impl PartialOrd for PageCharacter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.base_character.partial_cmp(&other.base_character)
    }
}

impl Eq for PageCharacter {
    
}

impl Ord for PageCharacter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.base_character.cmp(&other.base_character)
    }
}

// TODO: impl RitualCharacter