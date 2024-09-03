-- Your SQL goes here
ALTER TABLE experiences ADD COLUMN company_link TEXT NOT NULL;
ALTER TABLE experiences RENAME COLUMN position TO your_position;