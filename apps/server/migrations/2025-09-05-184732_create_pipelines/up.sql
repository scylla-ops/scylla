-- Your SQL goes here
CREATE TABLE pipelines
(
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content    TEXT         NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
