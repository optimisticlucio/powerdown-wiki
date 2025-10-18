
CREATE TABLE art (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.
    page_slug text NOT NULL UNIQUE,

    -- TODO: I used to have a "format" enum to say if this has a video or not. How to better handle that? Runtime check?

    creation_date date NOT NULL DEFAULT CURRENT_DATE,
    --TODO: I should have a "last modification date" for my own usage. How to make it update on read?
    --TODO: I should write whoever's been the last person to change this. NULL means it was sysadmin (me).
    title text NOT NULL,
    creators text[] NOT NULL CONSTRAINT has_creators CHECK (array_length(creators, 1) > 0), 

    thumbnail text NOT NULL, -- Assumed to be a link to the thumbnail file.

    tags text[],

    description text,

    nsfw boolean --TODO: Should we have other flags? This is clearly not a tag, it has unique behaviour.
);

CREATE TABLE art_file (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.
    belongs_to int
        REFERENCES art(id)
        ON DELETE CASCADE,
    
    file_url text NOT NULL,
    internal_ordering int NOT NULL -- Whether this image is the first, second, third, etc, in the given post.
)