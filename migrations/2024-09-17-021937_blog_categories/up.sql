-- Your SQL goes here

CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE blog_categories (
    blog_id INTEGER REFERENCES blogs(id),
    category_id INTEGER REFERENCES categories(id),
    PRIMARY KEY (blog_id, category_id)
);