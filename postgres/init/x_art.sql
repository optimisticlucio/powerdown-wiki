CREATE TYPE art_post_state AS ENUM (
    'public', -- Publicly viewable, standard state.
    'pending_approval', -- User-uploaded, pending admin review to be moved to public. Not visible.
    'processing' -- Currently mid-process by the server and/or database. Should not be viewable.
    );

CREATE TABLE art (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.
    page_slug text NOT NULL UNIQUE CHECK (TRIM(page_slug) != ''),

    -- TODO: I used to have a "format" enum to say if this has a video or not. How to better handle that? Runtime check?

    creation_date date NOT NULL DEFAULT CURRENT_DATE,
    --TODO: I should have a "last modification date" for my own usage. How to make it update on read?
    --TODO: I should write whoever's been the last person to change this. NULL means it was sysadmin (me).
    title text NOT NULL CHECK (TRIM(title) != ''),
    creators text[] NOT NULL CONSTRAINT has_creators CHECK (array_length(creators, 1) > 0), 

    thumbnail text NOT NULL, -- Assumed to be the key of the thumbnail in the public bucket.

    tags text[] NOT NULL DEFAULT ARRAY[]::text[],

    description text CHECK (TRIM(description) != ''),

    is_nsfw boolean NOT NULL DEFAULT FALSE, --TODO: Should we have other flags? This is clearly not a tag, it has unique behaviour.
    post_state art_post_state NOT NULL DEFAULT 'public',

    uploading_user_id integer -- If NULL, points to "Unknown User", to handle deleted accounts or pre-website archival.
        REFERENCES site_user(id)
        ON DELETE SET NULL
        DEFAULT NULL,

    CHECK (page_slug NOT IN ('new', 'add', 'update', 'null', '')) -- Make sure that we don't overlap with any hardcoded pages.
);

CREATE TABLE art_file (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.
    belongs_to int NOT NULL
        REFERENCES art(id)
        ON DELETE CASCADE,

    s3_key text NOT NULL, -- Points to the public bucket key
    internal_order int NOT NULL, -- Whether this image is the first, second, third, etc, in the given post.

    UNIQUE (belongs_to, internal_order)
);
