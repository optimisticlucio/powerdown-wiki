
pub struct BaseArt {
    db_id: i32,
    pub title: String,
    pub creators: Vec<String>,
    pub thumbnail_url: String,
    pub slug: String,
    pub has_video: bool
}

pub struct PageArt {
    pub base_art: BaseArt
}