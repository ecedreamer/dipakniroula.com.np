CREATE TABLE sessions
(
    id         SERIAL PRIMARY KEY,
    session_id TEXT     NOT NULL UNIQUE,
    user_id    TEXT  NOT NULL,
    data       TEXT,
    expires_at TIMESTAMP NOT NULL
);