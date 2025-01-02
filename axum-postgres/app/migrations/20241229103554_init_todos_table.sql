-- Add migration script here
CREATE TABLE IF NOT EXISTS todos (
    id UUID PRIMARY KEY,
    text TEXT NOT NULL,
    completed BOOLEAN NOT NULL
);
