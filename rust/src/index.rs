
struct FrontpageItem {
    pub name: String,
    pub url: String,
    pub image_url: String
}

pub async fn homepage() -> &'static str {
    "Huh, it worked!"
}
