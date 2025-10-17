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

    CHECK (page_slug NOT IN ('new', 'add', 'update', 'null', '')) -- Make sure that we don't overlap with any hardcoded pages.
);

CREATE TABLE ritual_info(
    character_id int PRIMARY KEY
        REFERENCES character(id)
        ON DELETE CASCADE,
    power_name text NOT NULL,
    power_description text NOT NULL
);