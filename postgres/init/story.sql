CREATE TABLE story (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.
    page_slug text NOT NULL UNIQUE CHECK (TRIM(page_slug) != ''),

    title text NOT NULL CHECK (TRIM(title) != ''),
    inpage_title text (TRIM(inpage_title) != ''),
    tagline text CHECK (TRIM(tagline) != ''),
    description text NOT NULL CHECK (TRIM(description) != ''),
    
    creators text[] NOT NULL CONSTRAINT has_creators CHECK (array_length(creators, 1) > 0), 

    creation_date date NOT NULL,

    last_updated timestamp with time zone DEFAULT NOW(), -- Don't overwrite this for the love of god.

    is_hidden boolean DEFAULT FALSE, -- Should this story be on the search and index page?

    custom_css text CHECK (TRIM(custom_css) != ''), -- Sanitize this on write and read!!!

    prev_story int DEFAULT NULL
        REFERENCES story(id)
        ON DELETE SET NULL,

    next_story int DEFAULT NULL
        REFERENCES story(id)
        ON DELETE SET NULL,

    editors_note text CHECK (TRIM(editors_note) != ''),

    content text NOT NULL CHECK (TRIM(content) != '') -- Assumed to be either markdown or HTML. Should I limit it somehow?

    CHECK (page_slug NOT IN ('new', 'add', 'update', 'null', '')) -- Make sure that we don't overlap with any hardcoded pages.
);

CREATE OR REPLACE FUNCTION update_last_updated()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW(); 
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_row_value_last_updated AFTER INSERT OR UPDATE ON story 
    FOR EACH ROW  -- This apparently means every row *in the transaction.* Not like, every row. Gave me a heart attack.
    EXECUTE FUNCTION update_last_updated();