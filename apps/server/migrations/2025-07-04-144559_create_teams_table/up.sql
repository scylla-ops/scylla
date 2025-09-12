-- Your SQL goes here
CREATE TABLE teams
(
    id         UUID PRIMARY KEY      DEFAULT gen_random_uuid(),
    name       VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);