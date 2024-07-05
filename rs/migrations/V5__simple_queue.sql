DROP TABLE QueuedTask;
DROP TABLE TaskResult;

CREATE TABLE QueuedTask(
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    created_at TEXT NOT NULL DEFAULT (DATETIME('now')),
    started_at TEXT,
    current_retry INTEGER NOT NULL DEFAULT 0,
    max_retry INTEGER NOT NULL,
    payload TEXT NOT NULL
);
