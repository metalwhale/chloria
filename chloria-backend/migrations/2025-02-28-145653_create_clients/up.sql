-- Your SQL goes here

CREATE TABLE clients (
    id SERIAL PRIMARY KEY,
    authentication_method TEXT NOT NULL,
    authentication_registry TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (authentication_method, authentication_registry)
);

CREATE TABLE client_credentials (
    id INT PRIMARY KEY REFERENCES clients,
    api_key TEXT NOT NULL,
    api_secret TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (api_key)
);
