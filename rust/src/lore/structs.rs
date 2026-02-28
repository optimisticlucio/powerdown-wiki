use postgres::Row;
use deadpool::managed::Object;
use deadpool_postgres::Manager;


#[derive(Debug)]
pub struct LoreCategory {
    pub id: i32,
    pub order_position: i32,

    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct BaseLore {
    pub id: i32,

    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct PageLore {
    pub base: BaseLore,

    pub parent_category: LoreCategory,
    
    pub content: String,
}

impl LoreCategory {
    /// Returns the lore pages which have this category as their parent.
    pub async fn get_associated_lore_bases(&self, db_connection: &Object<Manager>) -> Vec<BaseLore> {
        todo!()
    }

    // Given a LoreCategory ID, returns the relevant object.
    pub async fn get_by_id(db_connection: &Object<Manager>, given_id: i32) -> Option<Self> {
        todo!()
    }

    /// Returns all LoreCategories in the DB.
    pub async fn get_all_categories(db_connection: &Object<Manager>) -> Vec<Self>{
        todo!()
    }

    /// Converts a DB row with the relevant info to a LoreCategory struct.
    fn from_db_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            order_position: row.get("order_position"),
            title: row.get("title"),
            description: row.get("description")
        }
    }
}

impl BaseLore {
    /// Converts a DB row with the relevant info to a LoreBase struct.
    fn from_db_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description")
        }
    }
}

impl PageLore {
    /// Converts a DB row with the relevant info to a LorePage struct.
    async fn from_db_row(db_connection: &Object<Manager>, row: &Row) -> Self {
        Self {
            base: BaseLore::from_db_row(row),
            // This unwrap is fine bc db enforces that "belongs_to" exists
            parent_category: LoreCategory::get_by_id(db_connection, row.get("belongs_to")).await.unwrap(),
            content: row.get("content")
        }
    }

    /// Given a lore page slug, returns the relevant lore page. Returns none if the slug doesn't exist in the DB.
    pub async fn get_from_slug(db_connection: &Object<Manager>, slug: &str) -> Option<Self> {
        // TODO
        todo!()
    }
}