CREATE TABLE oauth_users (
    id         SERIAL PRIMARY KEY,
    google_id  TEXT NOT NULL UNIQUE,
    email      TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    avatar_url TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
