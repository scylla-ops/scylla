-- Tie each issued bearer token to the specific App secret that minted it. This
-- makes credential lifecycle immediate: revoking a secret cascades its tokens
-- away, and disabling a secret (or its app) lets the auth lookup reject the
-- token at once (the interceptor join filters on enabled + is_active).

ALTER TABLE app_tokens
    ADD COLUMN secret_id TEXT REFERENCES app_secrets (id) ON DELETE CASCADE;

-- Backfill existing tokens onto their app's earliest (default) secret.
UPDATE app_tokens t
SET secret_id = (
    SELECT s.id FROM app_secrets s
    WHERE s.app_id = t.app_id
    ORDER BY s.created_at ASC
    LIMIT 1
);

-- Drop any orphan token whose app somehow has no secret (shouldn't happen).
DELETE FROM app_tokens WHERE secret_id IS NULL;

ALTER TABLE app_tokens ALTER COLUMN secret_id SET NOT NULL;

CREATE INDEX app_tokens_secret_id_idx ON app_tokens (secret_id);
