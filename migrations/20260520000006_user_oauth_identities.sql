-- External identity links for OAuth login (e.g. GitHub). One row per
-- (provider, provider_user_id); a user may link several providers. The table
-- exists in every edition; only the `oauth-github` code path uses it.

CREATE TABLE user_oauth_identities (
    user_id          TEXT        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider         TEXT        NOT NULL,
    provider_user_id TEXT        NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (provider, provider_user_id)
);

CREATE INDEX user_oauth_identities_user_idx ON user_oauth_identities (user_id);
