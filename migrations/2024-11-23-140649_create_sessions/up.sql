CREATE TABLE sessions
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT     NOT NULL UNIQUE,
    user_id    TEXT  NOT NULL,
    data       TEXT,
    expires_at DATETIME NOT NULL
);