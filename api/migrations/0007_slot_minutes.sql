ALTER TABLE user_settings ADD COLUMN slot_minutes INTEGER
    CHECK (slot_minutes IS NULL OR slot_minutes >= 1);
