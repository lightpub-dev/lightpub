CREATE TABLE QueuedTask (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT (DATETIME('now')),
    scheduled_at TEXT NOT NULL,
    started_at TEXT,
    retry INTEGER DEFAULT 0,
    max_retry INTEGER NOT NULL,
    has_return INTEGER NOT NULL,
    payload TEXT NOT NULL
);

CREATE TABLE TaskResult (
    id INTEGER PRIMARY KEY,
    added_at TEXT DEFAULT (DATETIME('now')),
    body TEXT NOT NULL
);