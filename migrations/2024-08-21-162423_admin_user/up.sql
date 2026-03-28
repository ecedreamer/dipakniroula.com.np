-- Your SQL goes here
CREATE TABLE admin_users (
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    password TEXT NOT NULL
);