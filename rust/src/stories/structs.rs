use derive_builder::Builder;

#[derive(Clone, Builder)]
struct BaseStory {
    title: String,
    description: String,
    creators: Vec<String>,
    creation_date: chrono::NaiveDate,
    #[builder(default = false)]
    is_hidden: bool
}

#[derive(Clone, Builder)]
struct PageStory {
    base_story: BaseStory,
    #[builder(default = None)]
    inpage_title: Option<String>,
    #[builder(default = None)]
    tagline: Option<String>,
    tags: Vec<String>,
    #[builder(default = None)]
    previous_story: Option<BaseStory>,
    #[builder(default = None)]
    next_story: Option<BaseStory>,
    #[builder(default = None)]
    custom_css: Option<String>,
}