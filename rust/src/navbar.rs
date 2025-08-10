use askama::{Template};

#[derive(Template)] 
#[template(path = "components/navbar.html")]
pub struct Navbar {
    user: Option<String>
}

impl Navbar {
    pub fn not_logged_in() -> Navbar {
        Navbar { user: None }
    }
}