ALTER TABLE site_user
ADD COLUMN last_seen timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP;

-- If we update last_seen, we shouldn't update last_modified, really.
DROP TRIGGER user_info_last_modified ON site_user;
CREATE TRIGGER user_info_last_modified
BEFORE UPDATE ON site_user
FOR EACH ROW
WHEN ((to_jsonb(OLD) - 'last_seen' - 'last_modified_date')
        IS DISTINCT FROM (to_jsonb(NEW) - 'last_seen' - 'last_modified_date'))
EXECUTE FUNCTION update_last_modified_date();