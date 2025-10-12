use postgres::Row;
use postgres_types::{FromSql, ToSql, Type};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use derive_builder::Builder;
use serde::{Deserialize, Deserializer, de};

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
    #[builder(default = None)]
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub art_urls: Vec<String>,
    pub creation_date: chrono::NaiveDate,
}

impl BaseArt {
    /// Gets [amount_to_return] amount of art pieces, starting from the [index] newest piece.
    pub async fn get_art_from_index(db_connection: Object<Manager>, index: i64, amount_to_return: i64, search_parameters: &ArtSearchParameters) -> Vec<Self>{
        // TODO: Narrow down select so it runs faster.
        let mut query_parameters: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&amount_to_return, &index];

        let query_where = search_parameters.get_postgres_where(&mut query_parameters);

        let query = format!("SELECT * FROM art {} ORDER BY creation_date DESC LIMIT $1 OFFSET $2", query_where);

        let requested_art_rows = db_connection.query(
            &query,
            &query_parameters).await.unwrap();

        requested_art_rows.iter().map(Self::from_db_row).collect()
    }

    /// Converts a DB row with the relevant info to a BaseArt struct.
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

impl PageArt {
    pub async fn get_by_slug(db_connection: Object<Manager>, page_slug: &str) -> Option<Self> {
        let requested_art = db_connection.query_one(
            "SELECT * FROM art WHERE page_slug=$1",
            &[&page_slug]).await
            .ok()?;

        Some(Self::from_db_row(&requested_art))
    }

    /// Converts a DB row with the relevant info to a PageArt struct.
    fn from_db_row(row: &Row) -> Self {
        PageArt {
            base_art: BaseArt::from_db_row(row),
            description: row.get("description"),
            tags: row.try_get("tags").unwrap_or(Vec::new()),
            art_urls: row.get("files"),
            creation_date: row.get("creation_date")
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct ArtSearchParameters {
    #[serde(default = "default_page_number")]
    pub page: i64,

    #[serde(default, deserialize_with = "deserialize_tags")]
    pub tags: Vec<String>,

    #[serde(default)]
    pub nsfw: bool // TODO: Ignored in search?

    // TODO: Handle Artist Name
}

fn default_page_number() -> i64 {
    1
}

impl ArtSearchParameters {
    /// Creates the WHERE section of a postgresql statement for these parameters. Modifies a given set of function parameters.
    /// Lifetime of parameter modifications tied to lifetime of struct.
    pub fn get_postgres_where<'a>(&'a self, params: &mut Vec<&'a (dyn tokio_postgres::types::ToSql + Sync)>) -> String{
        let mut query_conditions: Vec<String> = Vec::new();

        if self.nsfw {
            query_conditions.push("nsfw".to_string());
        }
        else {
            query_conditions.push("NOT nsfw".to_string());
        }

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

    /// Returns how the parameter section of a URL with these parameters should look like. 
    pub fn to_uri_parameters(&self, include_page_number: bool) -> String {
        let mut parameters: Vec<String> = Vec::new();

        if include_page_number && self.page > 1 {
            parameters.push(format!("page={}", self.page));
        }

        if self.nsfw {
            parameters.push("nsfw=true".to_string());
        }

        if !self.tags.is_empty() {
            parameters.push(format!("tags={}", self.tags.join(",")));
        }

        // -- Return --

        if parameters.is_empty() {
            "".to_string()
        }
        else {
            format!("?{}", parameters.join("&"))
        }
    }
}

impl Default for ArtSearchParameters {
    fn default() -> Self {
        ArtSearchParameters { 
            page: default_page_number(),
            tags: Vec::new(),
            nsfw: false }
    }
}

/// Deserializes tags from a single string to vec<string>.
fn deserialize_tags<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // Expected format is a list of lowercase, numbers, and dashes, with a comma delimiter.
    let s = String::deserialize(deserializer)?;
    
    if s.is_empty() {
        return Ok(Vec::new());
    }
    
    Ok(s.split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect())
}