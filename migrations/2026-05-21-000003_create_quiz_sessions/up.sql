CREATE TABLE quiz_sessions (
    id SERIAL PRIMARY KEY,
    session_uuid TEXT NOT NULL UNIQUE,
    questions_json JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
