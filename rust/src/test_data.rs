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
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect(),
            page_contents: 
r#"

# Bio

Sky Starr is not an Electi.
That would require her to be human.

Skyber Wyrmond is an immortal, flamethrowing fragment of a dead god. Not a god. The god. Her existence is regarded as a myth to most, and a headache to the few aware of her true nature.

Skyber is the origin point. The primordial spark. The why behind every glowing hand, feral mutation, technology bubble all around Texas; Every Electi, Therian, and several other metaphysical anamolous hiccups that have driven science and religion in circles for centuries. She didn’t remember much of godhood, or any of it at all, before the Wildcard Emissary incident (the less said about that, the better) jogged something loose. Now she’s trying to rediscover herself one cautious step at a time in the form of Sky Starr, a student with a vaguely posh accent and an overly competitive attitude about card games.

Skyber is obsessed with the delicate dance between endless pleasure and heart-shattering perils at the whims of lady luck. She embodies the seductive allure and endless nights of Las Vegas, fueling her insatiable appetite for entertainment and purpose by stoking flames of passion in the hearts of humans. Despite having to keep a low profile nowadays, her insetiable apatite for entertainment and materialisim still slips through to her human disguise.

# Anomalies

**Eldritch Starfire -** Skyber's entire being is composed of eldtritch starfire, the substance stars were forged from. She can create and control any flame, which takes on a divine violet hue and burns hotter the closer it is to her. Starfire grants her vision and hearing through will-o-wisps acting as her eyes. Her body generates intense heat, capable of melting any metal.

**Living Legend -** Skyber is a true dragon with immense strength and stamina. Her nearly impenetrable scales are heavy, making her massive flight-capable wings even more terrifying. The sigil on her chest binds her to this world, ensuring her rebirth through another gateway if her physical form perishes.

**Minor Shapeshifting -** Unlike her more flexiable counterpart, Skyber is able to adjusted her flame-born composition but only slightly- like changing her hair color. It's only with Nova that Skyber can truly change her shape, and without their influence, exerting her powers is likely to undo any disguise and rebound Skyber to her true form.

**Transcendence -** Skyber is able to empower an Electi to re-focus their ability into it's true form, a godlike, reality bending power. However, Skyber can only sustain such focus for a short moment, leaving her entirely drained. This poses such unimaginable risks that even a gambling addict like Skyber would instantly fold on rather than ever consider it beyond a life or death situation. 

"#.to_owned()
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
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect(),
            page_contents:
r#"
# Bio

There is shockingly little to say about the sharp-dressed president of the Games and Recreation club. Sky somehow feels as if she’s just arrived at E.R.A. with her cousin, Noa, and at the same time, feels like they’ve always been there. When asked how long she’s been away from her home in London, she tends to answer something like “longer than you”.

No one knows what to make of her. Some admire her, some fear her. Most try to avoid her if possible, yet none can quite escape her mesmerizing presence, like a [star](/characters/skyber/) pulling on the heavenly bodies circling it till the end of time.

# Electi Ability: Dragonfire

Sky is able to conjure and manipulate abnormal violet flames that require no oxygen or fuel. That, alongside the horns adorning her head earned Sky the nickname of "Dragoness", which she takes in pride despite only being an African Horned Snake Therian. Sky's near mastery over her powers allows her to "conjure" extra limbs made of flames, most commonly a pair of wings that are miracously capable of flight, according to witness reports.
"#.to_owned()
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
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect(),
            page_contents:
r#"
# Bio

Preston’s parents immigrated from Ireland to Michigan while his mother was still pregnant believing that it would be a big opportunity, Preston was born shortly after. Although his parent’s business saw sales for nearly 2 years they started to go under continuing to scrape the bottom of the barrel for another 3 years they decided to make a last-ditch effort and sold what they could to move over to Los Angeles, things only slightly getting better. As Preston grew up and was looked down on by others, which only grew once his Electi ability formed, he grew an underdog persona slightly sparked by his love of boxing. 

Most of his pre-teen to early twenties was spent dealing in many low-mid level scams and heists with a group of friends who all shared having Electi abilities that affected the way they looked. Slowly gaining a reputation and the eyes of The House gang, he was recruited but not so long after he was caught and out into ERA after trying to pull off a heist solo with plans he stole from other members.

# Electi

Preston’s ability to name himself ‘eye-inventory’ is connected to his left eye socket, this ability has caused his left eye to fall out but I'm return the empty eye socket works as personal storage for Preston being able to deposit and withdraw anything he can fit through the socket while other people can deposit items into his inventory they aren't able to take things out. If at any point Preston exceeds the number of items he can fit into the inventory it will cause everything to come spilling out of his eye, resulting in a harsh headache on top of that he takes on the weight of anything inside the inventory. The science behind this isn't known if there even was Preston sure as hell wouldn't want to hear it.
"#.to_owned()
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
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect(),
            page_contents:
r#"
# Bio

Abigail is a former outlaw turned law enforcement in the technologically obsolete region in the Southern United States known as the Frontier. Her personality is rough, with a very aggressive demeanor about her. She drinks heavily and smokes frequently to cope with a tragic past. Although her persona and appearance are rugged and dissatisfied, she enjoys simplicity in nature and prefers to camp outdoors, playing her guitar by a small fire and pitched tent. She had been invited by ERA Academy to teach about the law, history, and life in the Frontier and what to expect upon visitation. She currently resides and does so reluctantly at the encouragement of her daughter Jesse, whom she consistently keeps in touch with.

# Electi Ability: Bullet Time

Abigail can achieve "Bullet Time", increasing her perception tenfold, to the point the world seems slow to her eyes for a few moments. Using this ability causes extreme cognitive strain, often leaving her in a drunken-like haze after a mere moment of use, and loss of consciousness entirely under further use.

"#.to_owned()
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
            ].into_iter().map(|(x,y)| (String::from(x), String::from(y))).collect(),
            page_contents:
r#"

# Bio
Melanie likes the students at ERA and wishes to protect them. She’s Goldman’s right-hand nurse.

# Electi
???
"#.to_owned()
        }
    ]
}

pub fn get_frontpage_quotes() -> Vec<String> {
    vec!["Because discord will not last forever, but random web forums from the 1980s will outlast us all.",
    "Discord? More like Pisscorp.",
    "The real Power Down was the HTML we cried over along the way.",
    "What do you mean <body> has a 8 px margin around itself by default?? WHY??",
    "I built an entire website before I unboxed an Unusual hat."
    ].into_iter().map(String::from).collect()
}

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