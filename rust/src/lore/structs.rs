use crate::user::UsermadePost;
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use postgres::Row;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoreCategory {
    #[serde(default)]
    pub id: i32,

    pub order_position: i32,

    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BaseLore {
    #[serde(default)]
    pub id: i32,

    pub title: String,
    pub description: Option<String>,

    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct PageLore {
    #[serde(flatten)]
    pub base: BaseLore,

    pub parent_category: LoreCategory,

    pub content: String,
}

impl LoreCategory {
    /// Returns the lore pages which have this category as their parent.
    pub async fn get_associated_lore_bases(
        &self,
        db_connection: &Object<Manager>,
    ) -> Vec<BaseLore> {
        let requested_lore_bases = db_connection
            .query(
                "SELECT * FROM lore WHERE belongs_to_category=$1",
                &[&self.id],
            )
            .await
            .unwrap();

        requested_lore_bases
            .iter()
            .map(BaseLore::from_db_row)
            .collect()
    }

    // Given a LoreCategory ID, returns the relevant object.
    pub async fn get_by_id(db_connection: &Object<Manager>, given_id: i32) -> Option<Self> {
        let requested_category = db_connection
            .query_one("SELECT * FROM lore_category WHERE id=$1", &[&given_id])
            .await
            .ok()?;

        Some(Self::from_db_row(&requested_category))
    }

    /// Returns all LoreCategories in the DB.
    pub async fn get_all_categories(db_connection: &Object<Manager>) -> Vec<Self> {
        let requested_category_rows = db_connection
            .query("SELECT * FROM lore_category", &[])
            .await
            .unwrap();

        requested_category_rows
            .iter()
            .map(Self::from_db_row)
            .collect()
    }

    /// Converts a DB row with the relevant info to a LoreCategory struct.
    fn from_db_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            order_position: row.get("order_position"),
            title: row.get("title"),
            description: row.get("description"),
        }
    }
}

impl PartialEq for LoreCategory {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for LoreCategory {}

impl PartialOrd for LoreCategory {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoreCategory {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order_position.cmp(&other.order_position)
    }
}

impl UsermadePost for LoreCategory {
    fn can_be_modified_by(&self, user: &crate::user::User) -> bool {
        user.user_type.permissions().can_modify_lore
    }
}

impl BaseLore {
    /// Converts a DB row with the relevant info to a LoreBase struct.
    fn from_db_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            slug: row.get("slug"),
        }
    }
}

impl UsermadePost for BaseLore {
    fn can_be_modified_by(&self, user: &crate::user::User) -> bool {
        user.user_type.permissions().can_modify_lore
    }
}

impl PageLore {
    /// Converts a DB row with the relevant info to a LorePage struct.
    async fn from_db_row(db_connection: &Object<Manager>, row: &Row) -> Self {
        Self {
            base: BaseLore::from_db_row(row),
            // This unwrap is fine bc db enforces that "belongs_to" exists
            parent_category: LoreCategory::get_by_id(db_connection, row.get("belongs_to"))
                .await
                .unwrap(),
            content: row.get("content"),
        }
    }

    /// Given a lore page slug, returns the relevant lore page. Returns none if the slug doesn't exist in the DB.
    pub async fn get_from_slug(db_connection: &Object<Manager>, slug: &str) -> Option<Self> {
        let requested_page = db_connection
            .query_one("SELECT * FROM lore WHERE slug=$1", &[&slug])
            .await
            .ok()?;

        Some(Self::from_db_row(db_connection, &requested_page).await)
    }
}

impl UsermadePost for PageLore {
    fn can_be_modified_by(&self, user: &crate::user::User) -> bool {
        self.base.can_be_modified_by(user)
    }
}
