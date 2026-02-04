CREATE TYPE user_type AS ENUM (
    'normal',
    'uploader',
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
    profile_picture_s3_key text CHECK (TRIM(profile_picture_s3_key) != '') DEFAULT NULL, -- If null, insert some default pfp. Points to the public bucket.
    last_modified_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP, -- Updates whenever this table is modified, see `update_last_modified_date()` below.
    creator_name text CHECK (TRIM(creator_name) != '') -- The string which refers to this user in art posts and such.
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


CREATE FUNCTION update_last_modified_date()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_modified_date = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_info_last_modified
BEFORE UPDATE ON site_user
FOR EACH ROW
EXECUTE FUNCTION update_last_modified_date();