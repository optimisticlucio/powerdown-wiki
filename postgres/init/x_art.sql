CREATE TYPE art_post_state AS ENUM (
    'public', -- Publicly viewable, standard state.
    'pending_approval', -- User-uploaded, pending admin review to be moved to public. Not visible.
    'processing' -- Currently mid-process by the server and/or database. Should not be viewable.
    );

CREATE TABLE art (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.
    page_slug text NOT NULL UNIQUE CHECK (TRIM(page_slug) != ''),

    creation_date date NOT NULL DEFAULT CURRENT_DATE,
    last_modified_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP, -- Updates whenever this table is modified, see `update_last_modified_date()` below.
    --TODO: I should write whoever's been the last person to change this. NULL means it was a deleted user.
    title text NOT NULL CHECK (TRIM(title) != ''),
    creators text[] NOT NULL CONSTRAINT has_creators CHECK (
        array_length(creators, 1) > 0 AND -- At least one creator
        NOT ('' = ANY(creators)) AND -- None of the creators are ''
        array_position(creators, NULL) IS NULL -- None of the creators are null
        ), 

    thumbnail text NOT NULL, -- Assumed to be the key of the thumbnail in the public bucket.

    -- Tags are assumed to all be lowercase. Not necessarily alphanumeric, but lowercase.
    tags text[] NOT NULL DEFAULT ARRAY[]::text[] CHECK (
        NOT ('' = ANY(tags)) AND -- None of the tags are ''
        array_position(tags, NULL) IS NULL -- None of the tags are null
    ),

    description text CHECK (TRIM(description) != ''),

    is_nsfw boolean NOT NULL DEFAULT FALSE, --TODO: Should we have other flags? This is clearly not a tag, it has unique behaviour.
    post_state art_post_state NOT NULL DEFAULT 'public',

    uploading_user_id integer -- If NULL, points to "Unknown User", to handle deleted accounts or pre-website archival.
        REFERENCES site_user(id)
        ON DELETE SET NULL
        DEFAULT NULL,

    CHECK (page_slug NOT IN ('new', 'add', 'update', 'null', '')) -- Make sure that we don't overlap with any hardcoded pages.
);

CREATE FUNCTION update_last_modified_date()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_modified_date = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER art_last_modified
BEFORE UPDATE ON art
FOR EACH ROW
EXECUTE FUNCTION update_last_modified_date();

CREATE TABLE art_file (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.
    belongs_to int NOT NULL
        REFERENCES art(id)
        ON DELETE CASCADE,

    s3_key text NOT NULL, -- Points to the public bucket key
    internal_order int NOT NULL, -- Whether this image is the first, second, third, etc, in the given post.

    UNIQUE (belongs_to, internal_order)
);
