-- Sessions cascade on user deletion: when a user is removed (or their account
-- is purged for GDPR), all their tokens vanish atomically. `delete_expired`
-- relies on expires_at being indexed for fast sweeps.

CREATE TABLE sessions (
    id              TEXT        PRIMARY KEY,
    token           TEXT        NOT NULL,
    user_id         TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    last_active_at  TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX sessions_token_key ON sessions (token);
CREATE INDEX sessions_user_id_idx ON sessions (user_id);
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at);
