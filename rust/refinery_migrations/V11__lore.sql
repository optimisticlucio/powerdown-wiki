CREATE TABLE lore_category (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.

    last_modified_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP, -- Updates whenever this row is modified, see `update_last_modified_date()`.

    title text NOT NULL CHECK (TRIM(text) != ''),
    description text CHECK (TRIM(description) != ''), -- Should be short

    order_position int NOT NULL UNIQUE -- The categories are listed in some order. This int orders them, and is zero indexed.
);

CREATE TRIGGER lore_category_last_modified
BEFORE UPDATE ON lore_category
FOR EACH ROW
EXECUTE FUNCTION update_last_modified_date();

CREATE TABLE lore (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY, -- Created by db, auto-increments.

    last_modified_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP, -- Updates whenever this row is modified, see `update_last_modified_date()`.

    belongs_to_category int NOT NULL
        REFERENCES lore_category(id)
        ON DELETE CASCADE, -- The category which this post belongs to.

    slug text NOT NULL UNIQUE CHECK (trim(slug) != ''),

    title text NOT NULL CHECK (TRIM(title) != ''),
    description text CHECK (TRIM(description) != ''), -- Should be short

    content text NOT NULL CHECK (trim(content) != '')
);

CREATE TRIGGER lore_last_modified
BEFORE UPDATE ON lore
FOR EACH ROW
EXECUTE FUNCTION update_last_modified_date();