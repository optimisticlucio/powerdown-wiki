CREATE TYPE infobox_row AS (
    title text,
    description text
);

CREATE TABLE character (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.

    post_state post_state NOT NULL DEFAULT 'public',
    last_modified_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP, -- Updates whenever this row is modified, see `update_last_modified_date()`.

    retirement_reason text CHECK (TRIM(retirement_reason) != ''), -- Assumed to be in Markdown format.
    is_hidden boolean NOT NULL DEFAULT FALSE, 
    page_slug text NOT NULL UNIQUE,
    is_main_character boolean NOT NULL DEFAULT FALSE, -- Each creator should have one main character, unrelated to story importance.

    short_name text NOT NULL CHECK (TRIM(short_name) != ''),
    long_name text CHECK (TRIM(long_name) != ''),
    subtitles text[] NOT NULL CONSTRAINT nonzero_subtitles CHECK (array_length(subtitles, 1) > 0),
    relevant_tag text CHECK (TRIM(relevant_tag) != ''),
    creator text NOT NULL CHECK (TRIM(creator) != ''),

    thumbnail text NOT NULL CHECK (TRIM(thumbnail) != ''), -- Assumed to be an S3 key

    infobox infobox_row[] NOT NULL DEFAULT ARRAY[]::infobox_row[],
    page_image text NOT NULL, -- Assumed to be an S3 key
    logo text CHECK (TRIM(logo) != ''), -- Assumed to be a URL
    overlay_css text CHECK (TRIM(overlay_css) != ''), -- Everything here goes inside a <style>.overlay { }</style> 
    custom_css text CHECK (TRIM(custom_css) != ''), -- If you wanna do something fancier than just edit overlay.

    birthday date,

    page_text text CHECK (TRIM(page_text) != '') -- Assumed to be in Markdown format.

    CHECK (page_slug NOT IN ('new', 'add', 'update', 'null', '')) -- Make sure that we don't overlap with any hardcoded pages.
);

CREATE TRIGGER character_last_modified
BEFORE UPDATE ON character
FOR EACH ROW
EXECUTE FUNCTION update_last_modified_date();

CREATE TABLE ritual_info(
    character_id int PRIMARY KEY
        REFERENCES character(id)
        ON DELETE CASCADE,
    power_name text NOT NULL CHECK (TRIM(power_name) != ''),
    power_description text NOT NULL CHECK (TRIM(power_description) != '')
);
