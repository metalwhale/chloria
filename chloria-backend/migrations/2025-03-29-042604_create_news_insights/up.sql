-- Your SQL goes here

CREATE TABLE news_insights (
    id INT PRIMARY KEY REFERENCES news,
    fields TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
