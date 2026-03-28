-- Your SQL goes here
CREATE TABLE experiences (
    id SERIAL PRIMARY KEY,
    company_name TEXT NOT NULL,
    position TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT,
    responsibility TEXT,
    skills TEXT
);