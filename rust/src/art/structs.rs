use postgres::Row;
use postgres_types::{FromSql, ToSql, Type};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use derive_builder::Builder;
use serde::{Deserialize, Deserializer, de};
use rand::{distr::Alphanumeric, Rng};

use crate::art;

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
    pub is_nsfw: bool
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
            is_nsfw: row.get("is_nsfw"),
        }
    }

    /// Gets an unused ID in the DB, by creating a temp object in the DB and extracting its ID.
    /// WARNING: Remember to clean up the temp object if you end up not using the given ID.
    pub async fn get_unused_id(db_connection: Object<Manager>) -> i32 {
        let random_page_slug: String = rand::rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect();

        // There's a very slight chance this operation panics on correct behaviour
        // bc it uses random strings. It should probably be fine, but I should fix this someday.
        let insert_operation_result = db_connection.query_one(
            "INSERT INTO art (is_hidden, page_slug, title, creators, thumbnail)
            VALUES (true, $1, 'TEMP', ARRAY['RNJesus'], '')
            RETURNING id", &[&random_page_slug]).await.unwrap();

        insert_operation_result.get(0) // id is int, which converts to i32.
    } 
}

impl PageArt {
    pub async fn get_by_slug(db_connection: Object<Manager>, page_slug: &str) -> Option<Self> {
        let requested_art = db_connection.query_one(
            "SELECT * FROM art WHERE page_slug=$1",
            &[&page_slug]).await
            .ok()?;

        Some(Self::from_db_row(db_connection, &requested_art).await)
    }

    /// Converts a DB row with the relevant info to a PageArt struct.
    async fn from_db_row(db_connection: Object<Manager>, row: &Row) -> Self {
        let art_id: i32 = row.get("id");

        // Get the relevant art URLs from the art_file table.
        let mut art_files = db_connection.query("SELECT * FROM art_file WHERE belongs_to=$1", &[&art_id])
                .await.unwrap_or(Vec::new())
                .iter().map(|row| {
                    let index: i32 = row.get("internal_order");
                    let url: String = row.get("file_url");

                    (index, url)
                }).collect::<Vec<_>>();
        
        art_files.sort();

        let art_urls = art_files.iter().map(|(_, x)| x.to_owned()).collect::<Vec<_>>(); 

        PageArt {
            base_art: BaseArt::from_db_row(row),
            description: row.get("description"),
            tags: row.try_get("tags").unwrap_or(Vec::new()),
            art_urls,
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
    pub is_nsfw: bool // TODO: Ignored in search?

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

        // Instead of putting "NOT is_hidden" everywhere and suffer the bug of "oops I forgot"
        // I just shoved it here.
        // If you ever don't need it... just manually remove it, don't make it easy to accidentally remove.
        query_conditions.push("NOT is_hidden".to_string());

        if self.is_nsfw {
            query_conditions.push("is_nsfw".to_string());
        }
        else {
            query_conditions.push("NOT is_nsfw".to_string());
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

        if self.is_nsfw {
            parameters.push("is_nsfw=true".to_string());
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

    /// Returns the URI of said parameters, except the NSFW value is flipped, and page count is dropped.
    /// Primarily for the "nsfw" toggle on the art index.
    pub fn flipped_nsfw_uri_params(&self) -> String {
        Self {
            is_nsfw: !self.is_nsfw,
            ..self.clone()
        }.to_uri_parameters(false)
    }
}

impl Default for ArtSearchParameters {
    fn default() -> Self {
        ArtSearchParameters { 
            page: default_page_number(),
            tags: Vec::new(),
            is_nsfw: false 
        }
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