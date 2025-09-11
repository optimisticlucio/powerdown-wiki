
CREATE TABLE art (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.

    -- TODO: I used to have a "format" enum to say if this has a video or not. How to better handle that? Runtime check?

    creation_date date DEFAULT CURRENT_DATE,
    --TODO: I should have a "last modification date" for my own usage. How to make it update on read?
    --TODO: I should write whoever's been the last person to change this. NULL means it was sysadmin (me).
    title text NOT NULL,
    creators text[] NOT NULL, --TODO: Is there a way to check there's at least one creator?

    thumbnail text NOT NULL, -- Assumed to be a link to the thumbnail file.
    files text[] NOT NULL, -- Assumed to be links to the relevant img/video.

    tags text[],

    nsfw boolean --TODO: Should we have other flags? This is clearly not a tag, it has unique behaviour.
);