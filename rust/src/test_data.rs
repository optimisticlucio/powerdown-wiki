pub fn get_frontpage_ads() -> Vec<String> {
    vec![
        "/static/img/ads/its-radical.gif",
        "/static/img/ads/jason.png",
        "/static/img/ads/Orbeez.jpg",
        "/static/img/ads/penny.png",
        "/static/img/ads/rabbl.jpg",
        "/static/img/ads/dodrinkos.png",
        "/static/img/ads/sock.png",
        "/static/img/ads/not-a-virus.png",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
