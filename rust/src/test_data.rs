use crate::characters::Character;

pub fn get_test_characters() -> Vec<Character> {
    vec![
        Character {
            is_hidden: true,
            archival_reason: None,
            name: "Skyber".to_owned(),
            long_name: None,
            subtitles: vec!["Electi Prime", "True Form", "Fallen Goddess", "#1 Plot Device", "ERA's Largest Ego", "Walking Corpse"].into_iter().map(String::from).collect(),
            img_url: "https://powerdown.wiki/assets/img/characters/page-imgs/skyber-page.png".to_owned(),
            thumbnail_url: "https://powerdown.wiki/assets/img/characters/thumbnails/skyber.png".to_owned(),
            infobox: vec![
                ("Name", "Skyber Wyrmond"),
                ("Age", "Unknown"),
                ("Height", "8ft"),
                ("Gender", "Female"),
                ("Birthday", "Unknown"),
                ("Deaths", "Three"),
                ("Calamities Caused", "At least twelve")
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect()
        },
        Character {
            is_hidden: false,
            archival_reason: None,
            name: "Sky".to_owned(),
            long_name: None,
            subtitles: vec!["Elusive Dragoness", "Scaled Pyromancer", "Violet Card Dealer", "Out of Your League", "Lady Luck"].into_iter().map(String::from).collect(),
            img_url: "https://powerdown.wiki/assets/img/characters/page-imgs/sky-page.png".to_owned(),
            thumbnail_url: "https://powerdown.wiki/assets/img/characters/thumbnails/sky.png".to_owned(),
            infobox: vec![
                ("Name", "Sky Starr"),
                ("Age", "22"),
                ("Height", "6'2\"ft / 187.96cm"),
                ("Gender", "Female (she/her)"),
                ("Birthday", "May 18th"),
                ("Card Decks Owned", "Too Many"),
                ("Current Hair Color", "White")
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect()
        },
        Character {
            is_hidden: false,
            archival_reason: None,
            name: "Preston".to_owned(),
            long_name: None,
            subtitles: vec!["Prick", "One-Eyed Rat", "Occulant Void"].into_iter().map(String::from).collect(),
            img_url: "https://powerdown.wiki/assets/img/art-archive/preston.png".to_owned(),
            thumbnail_url: "https://powerdown.wiki/assets/img/characters/thumbnails/preston.png".to_owned(),
            infobox: vec![
                ("Name", "Preston Puntur"),
                ("Age", "21"),
                ("Height", "6\""),
                ("Birthday", "August & 3rd"),
                ("Gender", "Male"),
                ("Times burned alive", "5"),
                ("Items in the eye", "Who's to know?")
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect()
        },
        Character {
            is_hidden: false,
            archival_reason: Some("While not officially retired from the project, the author remains in extended hiatus to focus on their mental health. For now, **Abigail is not to be included in current or future events.**".to_owned()),
            name: "Abigail".to_owned(),
            long_name: None,
            subtitles: vec!["Arachnid Cowgirl"].into_iter().map(String::from).collect(),
            img_url: "https://powerdown.wiki/assets/img/characters/page-imgs/abi.png".to_owned(),
            thumbnail_url: "https://powerdown.wiki/assets/img/characters/thumbnails/abigail.png".to_owned(),
            infobox: vec![
                ("Name", "Abigail Brookes"),
                ("Age", "Mid-30s"),
                ("Height", "6'0\" / 183cm"),
                ("Gender", "Female (she/her)"),
                ("Favorite Food", "Her Husband"),
                ("2nd Favorite Food", "Pork & Beans"),
                ("Haws yee'd", "Many")
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect()
        },
        Character {
            is_hidden: false,
            archival_reason: None,
            name: "Melanie".to_owned(),
            long_name: Some("Nurse Melanie".to_owned()),
            subtitles: vec!["Miss Mystery"].into_iter().map(String::from).collect(),
            img_url: "https://powerdown.wiki/assets/img/art-archive/nurse-melanie-transparent.png".to_owned(),
            thumbnail_url: "https://powerdown.wiki/assets/img/characters/thumbnails/melanie.png".to_owned(),
            infobox: vec![
                ("Name", "Melanie"),
                ("Age", "???"),
                ("Height", "4\'11\""),
                ("Gender", "Female"),
                ("Birthday", "???"),
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect()
        }
    ]
}