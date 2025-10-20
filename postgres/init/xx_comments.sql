CREATE TABLE art_comment (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.
    posting_time timestamp with time zone NOT NULL DEFAULT NOW(),

    under_post int NOT NULL-- The post this was commented on.
        REFERENCES art(id)
        ON DELETE CASCADE,
    
    poster int -- If NULL, points to "Unknown User", to handle deleted accounts and such.
        REFERENCES user(id)
        ON DELETE SET NULL,

    contents text NOT NULL
);