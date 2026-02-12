// A crate for stuff that the askama templates need to read/use.
use lazy_static::lazy_static;
use std::env;

// URLs for the navbar. Format is ("Title", "url").
pub const NAVBAR_URLS: [(&str, &str); 2] = [("Art", "/art"), ("Characters", "/characters")];

lazy_static! {
    pub static ref DEBUG_ENABLED: bool = env::var("DEBUG").is_ok();
}
