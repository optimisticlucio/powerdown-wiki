use postgres::Row;
use postgres_types::{FromSql, ToSql, Type};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use derive_builder::Builder;

#[derive(Clone, Builder)]
pub struct BaseArt {
    #[builder(default)]
    db_id: i32,
    pub title: String,
    pub creators: Vec<String>,
    pub thumbnail_url: String,
    pub slug: String,
    #[builder(default = false)]
    pub has_video: bool,
    #[builder(default = false)]
    pub nsfw: bool
}

#[derive(Clone, Builder)]
pub struct PageArt {
    pub base_art: BaseArt,
    pub description: String,
    pub tags: Vec<String>,
    pub art_urls: Vec<String>,
    pub creation_date: chrono::NaiveDate,
}

impl BaseArt {
    /// Gets [amount_to_return] amount of art pieces, starting from the [index] newest piece.
    pub async fn get_art_from_index(db_connection: Object<Manager>, index: u32, amount_to_return: u32) -> Vec<Self>{
        let requested_art_rows = db_connection.query(
            "SELECT * FROM art ORDER BY creation_date LIMIT $1 OFFSET $2",
            &[&amount_to_return, &index]).await.unwrap();

        requested_art_rows.iter().map(Self::from_db_row).collect()
    }

    /// Converts a DB row with the relevant info to a BaseCharacter struct.
    fn from_db_row(row: &Row) -> Self {
        BaseArt {
            db_id: row.get("id"),
            title: row.get("title"),
            creators: row.get("creators"),
            thumbnail_url: row.get("thumbnail"),
            slug: row.get("page_slug"),
            has_video: false, //TODO: Handle this somehow.
            nsfw: row.get("nsfw"),
        }
    }
}