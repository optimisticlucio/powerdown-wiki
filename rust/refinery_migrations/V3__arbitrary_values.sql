CREATE TABLE arbitrary_value (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments. Not needed actually, but required by postgres.
    last_modified_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP, -- Updates whenever this table is modified, see `update_last_modified_date()` below.
    item_key text NOT NULL UNIQUE,
    item_value text NOT NULL 
);

CREATE TRIGGER arbitrary_value_last_modified
BEFORE UPDATE ON arbitrary_value
FOR EACH ROW
EXECUTE FUNCTION update_last_modified_date();

INSERT INTO arbitrary_value (
    item_key, item_value
) VALUES (
    'discord_invite_url', ''
);
