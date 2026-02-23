use chrono::{DateTime, Utc};

struct LoreCategory {
    pub id: i32,
    pub order_position: i32,

    pub title: String,
    pub description: Option<String>,
}

struct LorePage {
    pub id: i32,

    pub parent_category: LoreCategory,

    pub title: String,
    pub description: Option<String>,
    pub content: String,
}
