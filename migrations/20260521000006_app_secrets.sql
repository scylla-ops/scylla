-- Named secrets for machine Apps. An App can hold several; authentication
-- accepts the plaintext of any *enabled* one (id + secret exchange). Only the
-- hash is stored; the plaintext is shown once at creation. Revoking deletes the
-- row; disabling keeps it but flips `enabled` off. Cascades on app deletion.

CREATE TABLE app_secrets (
    id          TEXT        PRIMARY KEY,
    app_id      TEXT        NOT NULL REFERENCES apps (id) ON DELETE CASCADE,
    label       TEXT        NOT NULL,
    secret_hash TEXT        NOT NULL,
    enabled     BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (app_id, label)
);

CREATE INDEX app_secrets_app_id_idx ON app_secrets (app_id);

-- Backfill: each existing app's single secret becomes its 'default' secret.
INSERT INTO app_secrets (id, app_id, label, secret_hash, enabled, created_at, updated_at)
SELECT gen_random_uuid()::text, id, 'default', secret_hash, TRUE, created_at, updated_at
FROM apps;

-- The secret now lives in app_secrets, never on the app row.
ALTER TABLE apps DROP COLUMN secret_hash;
