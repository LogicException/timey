ALTER TABLE user_settings ADD COLUMN default_task_id INTEGER
    CHECK (default_task_id IS NULL OR default_task_id >= 1);
