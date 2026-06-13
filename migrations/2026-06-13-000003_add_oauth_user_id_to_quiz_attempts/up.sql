ALTER TABLE quiz_attempts ADD COLUMN oauth_user_id INTEGER REFERENCES oauth_users(id);
