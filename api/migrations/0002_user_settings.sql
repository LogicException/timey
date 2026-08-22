CREATE TABLE user_settings (
    user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    work_start_minutes INTEGER NOT NULL,
    work_end_minutes INTEGER NOT NULL,
    CHECK (work_start_minutes >= 0),
    CHECK (work_end_minutes >= 0),
    CHECK (work_start_minutes < work_end_minutes)
);

INSERT INTO user_settings (user_id, work_start_minutes, work_end_minutes)
SELECT id, 450, 975 FROM users;
