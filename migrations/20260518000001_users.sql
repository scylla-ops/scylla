-- Users are identified by ULID strings (TEXT) to keep parity with the
-- application-generated `UserId`. Username uniqueness is enforced at the DB
-- so concurrent signups fail fast with a unique-violation, mapped to
-- DomainError::Conflict in the repo layer.

CREATE TABLE users (
    id              TEXT        PRIMARY KEY,
    username        TEXT        NOT NULL,
    password_hash   TEXT        NOT NULL,
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL
);

CREATE UNIQUE INDEX users_username_key ON users (username);
CREATE INDEX users_created_at_idx ON users (created_at DESC);
