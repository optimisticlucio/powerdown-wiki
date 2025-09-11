CREATE TABLE story (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.

    title text NOT NULL,
    tagline text,
    description text,
    creators text[] NOT NULL, -- TODO: Assure not empty

    creation_date date NOT NULL,
    -- TODO: Figure out how to do "last modification date"

    is_hidden boolean DEFAULT FALSE, -- Should this story be on the search and index page?

    custom_css text, -- Sanitize this on write and read!!!

    /* TODO:
    - previous chapter
    - next chapter
    - audio readings?
    */

    editors_note text,

    content text NOT NULL -- Assumed to be either markdown or HTML. Should I limit it somehow?
);