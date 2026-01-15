CREATE TYPE user_type AS ENUM (
    'normal',
    'admin',
    'superadmin'
);

CREATE TYPE oauth_provider AS ENUM (
    'discord',
    'google',
    'github'
);

CREATE TABLE site_user (
    id integer PRIMARY KEY, -- GENERATE THESE AS RANDOM INTEGERS IN CODE!

    display_name text NOT NULL CHECK (TRIM(display_name) != ''),
    user_type user_type NOT NULL DEFAULT 'normal',
    profile_picture_s3_key text CHECK (TRIM(profile_picture_s3_key) != '') DEFAULT NULL -- If null, insert some default pfp. Points to the public bucket.
    -- TODO: Think of relevant fields.
);

-- Associations between users and OAuth2 providers
CREATE TABLE user_oauth_association (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,

    user_id integer NOT NULL
        REFERENCES site_user(id)
        ON DELETE CASCADE,

    provider oauth_provider NOT NULL,

    oauth_user_id text NOT NULL, -- The user ID, or equivalent thereof, on the provider's DB

    UNIQUE(provider, user_id),
    UNIQUE(provider, oauth_user_id)
);

CREATE TABLE user_session (
    user_id integer NOT NULL
        REFERENCES site_user(id)
        ON DELETE CASCADE,

    session_id text PRIMARY KEY, -- Session ID should be a long, random string.

    creation_time timestamp with time zone NOT NULL DEFAULT NOW() -- For the love of god don't set this manually.
    -- The server, whenever reading the session, should check if it's been enough time since the creation for the session to be invalid. If it is, delete the entry.

);
