-- Your SQL goes here
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    full_name TEXT NOT NULL,
    email TEXT NOT NULL,
    mobile TEXT,
    subject TEXT NOT NULL,
    message TEXT NOT NULL,
    date_sent TEXT NOT NULL
);