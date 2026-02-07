CREATE FUNCTION update_last_modified_date()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_modified_date = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TYPE post_state AS ENUM (
    'public', -- Publicly viewable, standard state.
    'pending_approval', -- User-uploaded, pending admin review to be moved to public. Not visible.
    'processing' -- Currently mid-process by the server and/or database. Should not be viewable.
);