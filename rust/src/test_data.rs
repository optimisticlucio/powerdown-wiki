pub fn get_frontpage_ads() -> Vec<String> {
    vec!["/assets/img/ads/its-radical.gif",
    "/assets/img/ads/jason.png",
    "/assets/img/ads/Orbeez.jpg",
    "/assets/img/ads/penny.png",
    "/assets/img/ads/rabbl.jpg",
    "/assets/img/ads/dodrinkos.png",
    "/assets/img/ads/sock.png",
    "/assets/img/ads/not-a-virus.png"
    ].into_iter().map(String::from).collect()
}