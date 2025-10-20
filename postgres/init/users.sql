CREATE TYPE user_type AS ENUM (
    'normal',
    'admin',
    'superadmin'
);

CREATE TABLE site_user (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- TODO: Should this be random IDs? Identity is sequential.

    username text NOT NULL CHECK (TRIM(username) != ''),
    type user_type NOT NULL DEFAULT 'normal'
    -- TODO: Add pfp.

    -- TODO: Think of relevant fields.
    -- TODO: Add OpenID stuff for login.
);