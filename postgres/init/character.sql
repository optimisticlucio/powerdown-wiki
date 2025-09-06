CREATE TYPE infobox_row AS (
    title text,
    description text
);

CREATE TABLE character (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.

    retirement_reason text, -- Assumed to be in Markdown format.
    is_hidden boolean NOT NULL DEFAULT FALSE, 
    page_slug text NOT NULL UNIQUE,

    short_name text NOT NULL,
    long_name text,
    subtitles text[] NOT NULL,
    relevant_tag text,
    creator text NOT NULL,

    thumbnail text NOT NULL, -- Assumed to be a URL

    infobox infobox_row[],
    page_image text NOT NULL, -- Assumed to be a URL
    logo text, -- Assumed to be a URL
    overlay_css text, -- Everything here goes inside a <style>.overlay { }</style> 
    custom_css text, -- If you wanna do something fancier than just edit overlay.

    birthday date,

    page_text text -- Assumed to be in Markdown format.
);

-- TODO: Create a Ritual table

INSERT INTO character(is_hidden, creator, page_slug, 
    thumbnail,
    short_name, subtitles,
    infobox,
    page_image, logo, overlay_css,
    page_text)
VALUES(TRUE, 'Sir Skyber', 'skyber',
    'https://powerdown.wiki/assets/img/characters/thumbnails/skyber.png',
    'Skyber', ARRAY['Electi Prime', 'True Form', 'Fallen Goddess', '#1 Plot Device', 'ERA''s Largest Ego', 'Walking Corpse'],
    ARRAY[
        ('Name', 'Skyber Wyrmond')::infobox_row,
        ('Age', 'Unknown')::infobox_row,
        ('Height', '8ft')::infobox_row,
        ('Gender', 'Female')::infobox_row,
        ('Birthday', 'Unknown')::infobox_row,
        ('Deaths', 'Three')::infobox_row,
        ('Calamities Caused', 'At least twelve')::infobox_row
    ],
    'https://powerdown.wiki/assets/img/characters/page-imgs/skyber-page.png', 'https://powerdown.wiki/assets/img/characters/logos/Skyber.png', 'mix-blend-mode: overlay; background: #453663;',
    '# Bio

Sky Starr is not an Electi.
That would require her to be human.

Skyber Wyrmond is an immortal, flamethrowing fragment of a dead god. Not a god. The god. Her existence is regarded as a myth to most, and a headache to the few aware of her true nature.

Skyber is the origin point. The primordial spark. The why behind every glowing hand, feral mutation, technology bubble all around Texas; Every Electi, Therian, and several other metaphysical anamolous hiccups that have driven science and religion in circles for centuries. She didn’t remember much of godhood, or any of it at all, before the Wildcard Emissary incident (the less said about that, the better) jogged something loose. Now she’s trying to rediscover herself one cautious step at a time in the form of Sky Starr, a student with a vaguely posh accent and an overly competitive attitude about card games.

Skyber is obsessed with the delicate dance between endless pleasure and heart-shattering perils at the whims of lady luck. She embodies the seductive allure and endless nights of Las Vegas, fueling her insatiable appetite for entertainment and purpose by stoking flames of passion in the hearts of humans. Despite having to keep a low profile nowadays, her insetiable apatite for entertainment and materialisim still slips through to her human disguise.

# Anomalies

**Eldritch Starfire -** Skyber''s entire being is composed of eldtritch starfire, the substance stars were forged from. She can create and control any flame, which takes on a divine violet hue and burns hotter the closer it is to her. Starfire grants her vision and hearing through will-o-wisps acting as her eyes. Her body generates intense heat, capable of melting any metal.

**Living Legend -** Skyber is a true dragon with immense strength and stamina. Her nearly impenetrable scales are heavy, making her massive flight-capable wings even more terrifying. The sigil on her chest binds her to this world, ensuring her rebirth through another gateway if her physical form perishes.

**Minor Shapeshifting -** Unlike her more flexiable counterpart, Skyber is able to adjusted her flame-born composition but only slightly- like changing her hair color. It''s only with Nova that Skyber can truly change her shape, and without their influence, exerting her powers is likely to undo any disguise and rebound Skyber to her true form.

**Transcendence -** Skyber is able to empower an Electi to re-focus their ability into it''s true form, a godlike, reality bending power. However, Skyber can only sustain such focus for a short moment, leaving her entirely drained. This poses such unimaginable risks that even a gambling addict like Skyber would instantly fold on rather than ever consider it beyond a life or death situation. 
');

INSERT INTO character(creator, page_slug, birthday,
    thumbnail,
    short_name, subtitles,
    infobox,
    page_image, logo, overlay_css,
    page_text)
VALUES('Sir Skyber', 'ridley', '2000-09-06',
    'https://powerdown.wiki/assets/img/characters/thumbnails/ridley.png',
    'Ridley', ARRAY['Lives To Instantly Regret It', 'Reads One Piece', 'Lives in a Cupboard', 'ERA''s Premier Manfailure', 'Needs a Break', 'Pokemon Aficionado', 'Missing Weezer Member'],
    ARRAY[
        ('Name', 'Ridley Ka''dhori')::infobox_row,
        ('Age', '18')::infobox_row,
        ('Height', '5''6" / 167 cm')::infobox_row,
        ('Gender', 'Male (he/him)')::infobox_row,
        ('Birthday', '6th of September')::infobox_row,
        ('Favorite Pokémon', 'Houndoom')::infobox_row,
        ('Friends', 'Like 6')::infobox_row
    ],
    'https://powerdown.wiki/assets/img/characters/page-imgs/ridley-page.png', 'https://powerdown.wiki/assets/img/characters/logos/Ridley.png', 'mix-blend-mode: overlay; background: linear-gradient(180deg, #948CFF 0%, #FF4D85 100%)',
    '# Bio

Nobody at E.R.A. knew who Ridley was until they suddenly really did—specifically, when he accidentally flooded the academy''s entire underground water system with several metric tons of highly absorbent polymer spheres. Or at least, some people know about him rather than none.

Ridley is a timid recluse whose life functions more like a cosmic joke than a narrative arc. He is not particularly gifted, particularly confident, or particularly lucky. In fact, he seems to possess the rare and deeply unfortunate superpower of being exactly where catastrophe is about to happen. Sometimes causing it. Sometimes being it. And that''s on top of already being an Electi.

Stuck in the guard barracks under permanent house arrest, he shuffles through life with the enthusiasm of a man dreading the next piano to drop, surviving by sheer, highly trained reflex and the occasional misplaced act of kindness.


# Electi

Ridley can replicate any given item given a reference and the base materials required to make a duplicate.

Replication requires intimate knowledge of the object being replicated. Unfortunately for Ridley, he sucks at it. He is often seen wearing his personal project, a warped second pair of glasses, atop his head.'

);