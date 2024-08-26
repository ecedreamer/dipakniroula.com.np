-- This file should undo anything in `up.sql`
ALTER TABLE blogs
DROP COLUMN published_date;

ALTER TABLE blogs
DROP COLUMN modified_date;

ALTER TABLE blogs
DROP COLUMN view_count;

ALTER TABLE blogs
DROP COLUMN is_active;