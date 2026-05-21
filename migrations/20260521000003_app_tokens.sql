-- Bearer tokens issued to machine Apps in exchange for their secret. Kept
-- separate from user `sessions`; the auth interceptor resolves a token here to
-- an App principal. Cascade on app deletion. expires_at is indexed for sweeps.

CREATE TABLE app_tokens (
    id          TEXT        PRIMARY KEY,
    token       TEXT        NOT NULL,
    app_id      TEXT        NOT NULL REFERENCES apps (id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX app_tokens_token_key ON app_tokens (token);
CREATE INDEX app_tokens_app_id_idx ON app_tokens (app_id);
CREATE INDEX app_tokens_expires_at_idx ON app_tokens (expires_at);
