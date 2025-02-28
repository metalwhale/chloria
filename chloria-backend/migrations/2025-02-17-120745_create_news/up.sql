-- Your SQL goes here

CREATE TABLE news (
    id SERIAL PRIMARY KEY,
    source_name TEXT NOT NULL,
    article_id TEXT NOT NULL,
    link TEXT,
    title TEXT,
    short_text TEXT,
    long_text TEXT,
    image_path TEXT,
    published_time TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (source_name, article_id)
);
