-- Add future columns: auth_provider/oidc_subject already present for optional OIDC.

CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password_hash TEXT,
    auth_provider TEXT NOT NULL DEFAULT 'local' CHECK (auth_provider IN ('local', 'oidc')),
    oidc_subject TEXT UNIQUE,
    role TEXT NOT NULL CHECK (role IN ('admin', 'user')),
    disabled INTEGER NOT NULL DEFAULT 0 CHECK (disabled IN (0, 1)),
    created_at TEXT NOT NULL
);

CREATE TABLE sessions (
    token_hash TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TEXT NOT NULL
);

CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_by INTEGER NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL
);

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL COLLATE NOCASE,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at TEXT NOT NULL,
    UNIQUE (user_id, name)
);

CREATE TABLE aufgaben (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL COLLATE NOCASE,
    archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    created_at TEXT NOT NULL,
    UNIQUE (user_id, name)
);

CREATE TABLE entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES tasks(id),
    project_id INTEGER REFERENCES projects(id),
    aufgabe_id INTEGER REFERENCES aufgaben(id),
    start_at TEXT NOT NULL,
    end_at TEXT,
    status TEXT NOT NULL CHECK (status IN ('running', 'complete', 'needs_task')),
    created_at TEXT NOT NULL
);

CREATE TABLE work_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    local_date TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('running', 'paused', 'stopped')),
    created_at TEXT NOT NULL
);

CREATE TABLE work_session_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    work_session_id INTEGER NOT NULL REFERENCES work_sessions(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('started', 'paused', 'resumed', 'stopped')),
    at TEXT NOT NULL
);

CREATE INDEX idx_sessions_user ON sessions (user_id);
CREATE INDEX idx_entries_user_start ON entries (user_id, start_at);
CREATE INDEX idx_tasks_user ON tasks (user_id);
CREATE INDEX idx_aufgaben_user ON aufgaben (user_id);
CREATE INDEX idx_work_sessions_user_date ON work_sessions (user_id, local_date);
CREATE INDEX idx_work_events_session ON work_session_events (work_session_id);
