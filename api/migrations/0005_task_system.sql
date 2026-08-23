ALTER TABLE tasks ADD COLUMN is_system INTEGER NOT NULL DEFAULT 0 CHECK (is_system IN (0, 1));

CREATE UNIQUE INDEX tasks_one_system_per_user ON tasks(user_id) WHERE is_system = 1;

UPDATE tasks SET is_system = 1 WHERE name = 'unbestimmt' COLLATE NOCASE;

INSERT INTO tasks (user_id, name, archived, is_system, created_at)
SELECT u.id, 'unbestimmt', 0, 1, u.created_at
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM tasks t WHERE t.user_id = u.id AND t.is_system = 1
);
