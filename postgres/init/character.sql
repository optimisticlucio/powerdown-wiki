CREATE TYPE infobox_row AS (
    title text,
    description text
);

CREATE TABLE character (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.

    retirement_reason text, -- Assumed to be in Markdown format.
    is_hidden boolean NOT NULL DEFAULT FALSE, 
    page_slug text NOT NULL UNIQUE,
    is_main_character boolean NOT NULL DEFAULT FALSE, -- Each creator should have one main character, unrelated to story importance.

    short_name text NOT NULL,
    long_name text,
    subtitles text[] NOT NULL CONSTRAINT nonzero_subtitles CHECK (array_length(subtitles, 1) > 0),
    relevant_tag text,
    creator text NOT NULL,

    thumbnail text NOT NULL, -- Assumed to be a URL

    infobox infobox_row[] NOT NULL DEFAULT ARRAY[]::infobox_row[],
    page_image text NOT NULL, -- Assumed to be a URL
    logo text, -- Assumed to be a URL
    overlay_css text, -- Everything here goes inside a <style>.overlay { }</style> 
    custom_css text, -- If you wanna do something fancier than just edit overlay.

    birthday date,

    page_text text -- Assumed to be in Markdown format.
);

CREATE TABLE ritual_info(
    character_id int PRIMARY KEY
        REFERENCES character(id)
        ON DELETE CASCADE,
    power_name text NOT NULL,
    power_description text NOT NULL
);

/*
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

INSERT INTO ritual_info (
    character_id,
    power_name,
    power_description
)
VALUES (
    (SELECT id FROM character WHERE page_slug = 'ridley'),
    'Dupe-licate',
    'Can create replicas of any item. The replicas depend on intimate knowledge of the item being replicated, and typically this results in an imperfect clone because he sucks at it.'
);

INSERT INTO character(creator, page_slug, is_main_character,
    retirement_reason,
    thumbnail,
    short_name, subtitles,
    infobox,
    page_image, logo, overlay_css,
    page_text)
VALUES (
    'Prushy', 'abigail', TRUE,
    'While not officially retired from the project, the author remains in extended hiatus to focus on their mental health. For now, **Abigail is not to be included in current or future events.**',
    'https://powerdown.wiki/assets/img/characters/thumbnails/abigail.png',
    'Abigail', ARRAY['Arachnid Cowgirl'],
    ARRAY[
        ('Name', 'Abigail Brookes')::infobox_row,
        ('Age', 'Mid-30''s')::infobox_row,
        ('Height', '6''0" / 183cm')::infobox_row,
        ('Gender', 'Female (she/her)')::infobox_row,
        ('Favorite Food', 'Her husband')::infobox_row,
        ('2nd Favorite Food', 'Pork n'' beans')::infobox_row,
        ('Haws yee''d', 'Many')::infobox_row
    ],
    'https://powerdown.wiki/assets/img/characters/page-imgs/abi.png', 'https://powerdown.wiki/assets/img/characters/logos/Abigail.png', 'mix-blend-mode: overlay; background: linear-gradient(180deg, #631209 0%, #754b36 100%);',
    '# Bio

Abigail is a former outlaw turned law enforcement in the technologically obsolete region in the Southern United States known as the Frontier. Her personality is rough, with a very aggressive demeanor about her. She drinks heavily and smokes frequently to cope with a tragic past. Although her persona and appearance are rugged and dissatisfied, she enjoys simplicity in nature and prefers to camp outdoors, playing her guitar by a small fire and pitched tent. She had been invited by ERA Academy to teach about the law, history, and life in the Frontier and what to expect upon visitation. She currently resides and does so reluctantly at the encouragement of her daughter Jesse, whom she consistently keeps in touch with.

# Electi Ability: Bullet Time

Abigail can achieve "Bullet Time", increasing her perception tenfold, to the point the world seems slow to her eyes for a few moments. Using this ability causes extreme cognitive strain, often leaving her in a drunken-like haze after a mere moment of use, and loss of consciousness entirely under further use.'
);

INSERT INTO ritual_info (
    character_id,
    power_name,
    power_description
)
VALUES (
    (SELECT id FROM character WHERE page_slug = 'abigail'),
    'Bullet Time',
    'Time seems to slow in her perception, allowing quicker reaction. Causes extreme cognitive strain.'
);

INSERT INTO character(creator, page_slug, is_main_character,
    thumbnail,
    short_name, subtitles,
    infobox,
    birthday,
    page_image, logo, overlay_css,
    page_text)
VALUES (
    'HivemindHypnosis', 'gabriel', TRUE,
    'https://powerdown.wiki/assets/img/characters/thumbnails/gabriel.png',
    'Gabriel', ARRAY['Ento-Freak'],
    ARRAY[
        ('Name', 'Gabriel Torres')::infobox_row,
        ('Age', '20 years old')::infobox_row,
        ('Height', '5''10"')::infobox_row,
        ('Gender', 'Male')::infobox_row,
        ('Birthday', 'October 24th')::infobox_row,
        ('Favorite Food', 'Corn Beef Hash')::infobox_row,
        ('Favorite Color', 'Orange')::infobox_row,
        ('Least Favorite Food', 'Mango (Allergic)')::infobox_row,
        ('Kids Killed', '1-ish')::infobox_row
    ],
    '2000-10-24',
    'https://powerdown.wiki/assets/img/art-archive/judgy-gabriel.png', 'https://powerdown.wiki/assets/img/characters/logos/Gabe.png', 'mix-blend-mode: overlay; background: #EC9706;',
    '# Bio
A young kid born in Argentina. Many of his family live has been kept scarce due to him not remembering much as a child. Due to his Electi Ability causing him to freak out and push a fellow peer down the stairs, paralyzing them for life, Gabriel was sentenced to an Electi Juvenile Detention Center until E.R.A. acquired him for rehabilitation. Since then, he has gotten into fights with multiple students and plots the demise of one of them. He is a cold individual with a lot of problems, but who isn''t nowadays?  

# Electi Ability - Entopsychosis
The user is able to understand arthropods, however he cannot communicate back to them, but only command certain groups with different telepathic signals similar to chemical scents used for differentiating insect tribes from one another. However, this means that any insect that senses the different chemical scent will instantly become hostile towards the ones he is controlling, and attack if provoked.

Gabriel has been able to have a one-way communication with many arthropods, insects mostly as they are the ones he sees most of the time. The commands he is able to give are mostly only a few words since insect comprehension is much more simple than human understanding. However, despite this he much more connected to them in a sympathetic light. Though, it has made him much less sympathetic towards humans. 

Gabriel''s Electi Ability allow him to be offensive in swarming enemies while also having capabilities to alter terrain, give status conditions, or gather information of the locations of others. Though, it does come with the issue of having multiple hives want to literally kill each other.'
);

INSERT INTO ritual_info (
    character_id,
    power_name,
    power_description
)
VALUES (
    (SELECT id FROM character WHERE page_slug = 'gabriel'),
    'Entopsychosis',
    'Can talk to bugs, but they can''t talk back to him.'
);*/