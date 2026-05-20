-- Email for login-by-email, transactional mail and OAuth linking. Nullable so
-- existing username-only accounts stay valid; the application requires it at
-- signup. Partial unique index lets many NULLs coexist while keeping addresses
-- unique among accounts that have one.

ALTER TABLE users ADD COLUMN email TEXT;

CREATE UNIQUE INDEX users_email_key ON users (email) WHERE email IS NOT NULL;
