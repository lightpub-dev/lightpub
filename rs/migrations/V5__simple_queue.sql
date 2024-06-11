DROP TABLE QueuedTask;
DROP TABLE TaskResult;

CREATE TABLE QueuedTask(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT (DATETIME('now')),
    started_at TEXT,
    current_retry INTEGER DEFAULT 0,
    max_retry INTEGER NOT NULL,
    payload TEXT NOT NULL
);
