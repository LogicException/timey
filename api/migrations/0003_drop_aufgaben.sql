INSERT OR IGNORE INTO tasks (user_id, name, archived, created_at)
SELECT e.user_id,
       substr(t.name || ' ' || a.name, 1, 120),
       0,
       '2026-08-23T00:00:00Z'
FROM entries e
JOIN tasks t ON t.id = e.task_id
JOIN aufgaben a ON a.id = e.aufgabe_id
WHERE e.task_id IS NOT NULL AND e.aufgabe_id IS NOT NULL
GROUP BY e.user_id, substr(t.name || ' ' || a.name, 1, 120);

UPDATE entries
SET task_id = (
    SELECT t2.id
    FROM tasks t2
    JOIN tasks t1 ON t1.id = entries.task_id
    JOIN aufgaben a ON a.id = entries.aufgabe_id
    WHERE t2.user_id = entries.user_id
      AND t2.name = substr(t1.name || ' ' || a.name, 1, 120)
)
WHERE task_id IS NOT NULL AND aufgabe_id IS NOT NULL;

CREATE TABLE entries_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES tasks(id),
    project_id INTEGER REFERENCES projects(id),
    start_at TEXT NOT NULL,
    end_at TEXT,
    status TEXT NOT NULL CHECK (status IN ('running', 'complete', 'needs_task')),
    created_at TEXT NOT NULL
);

INSERT INTO entries_new (id, user_id, task_id, project_id, start_at, end_at, status, created_at)
SELECT id, user_id, task_id, project_id, start_at, end_at, status, created_at FROM entries;

DROP TABLE entries;

ALTER TABLE entries_new RENAME TO entries;

CREATE INDEX idx_entries_user_start ON entries (user_id, start_at);

DROP TABLE aufgaben;
