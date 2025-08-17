
#[derive(Clone)]
pub struct Character {
    pub is_hidden: bool,
    pub archival_reason: Option<String>, // If none, not archived.

    pub name: String,
    pub long_name: Option<String>,
    pub subtitles: Vec<String>,
    pub author: String,
    pub logo_url: Option<String>,
    // TODO: character birthday
    pub thumbnail_url: String,
    pub img_url: String,
    pub infobox: Vec<(String, String)>,
    // TODO: relationships?
    pub overlay_css: Option<String>,
    pub page_contents: String,
}

// TODO: Get character ritual info