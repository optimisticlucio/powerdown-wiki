CREATE TABLE story (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.

    title text NOT NULL CHECK (TRIM(title) != ''),
    tagline text CHECK (TRIM(tagline) != ''),
    description text CHECK (TRIM(description) != ''),
    creators text[] NOT NULL CONSTRAINT has_creators CHECK (array_length(creators, 1) > 0), 

    creation_date date NOT NULL,

    last_updated timestamp with time zone DEFAULT NOW(),

    is_hidden boolean DEFAULT FALSE, -- Should this story be on the search and index page?

    custom_css text CHECK (TRIM(custom_css) != ''), -- Sanitize this on write and read!!!

    /* TODO:
    - previous chapter
    - next chapter
    - audio readings?
    */

    editors_note text CHECK (TRIM(editors_note) != ''),

    content text NOT NULL CHECK (TRIM(content) != '') -- Assumed to be either markdown or HTML. Should I limit it somehow?
);

CREATE OR REPLACE FUNCTION update_last_updated()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW(); 
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_row_value_last_updated AFTER UPDATE INSERT OR UPDATE ON story 
    FOR EACH ROW  -- This apparently means every row *in the transaction.* Not like, every row. Gave me a heart attack.
    EXECUTE FUNCTION update_last_updated();