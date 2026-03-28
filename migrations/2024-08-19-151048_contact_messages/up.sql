-- Your SQL goes here
CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    full_name TEXT NOT NULL,
    email TEXT NOT NULL,
    mobile TEXT,
    subject TEXT NOT NULL,
    message TEXT NOT NULL,
    date_sent TEXT NOT NULL
);