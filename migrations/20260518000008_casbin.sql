-- Schema expected by the `sqlx-adapter` crate (casbin-rs). The adapter would
-- create this table on first use at runtime, but we materialise it here so
-- compile-time `sqlx::query!` checks in `sqlx-adapter` succeed when the
-- workspace is built against this DB or the offline `.sqlx/` cache.

CREATE TABLE casbin_rule (
    id      SERIAL  PRIMARY KEY,
    ptype   VARCHAR NOT NULL,
    v0      VARCHAR NOT NULL,
    v1      VARCHAR NOT NULL,
    v2      VARCHAR NOT NULL,
    v3      VARCHAR NOT NULL,
    v4      VARCHAR NOT NULL,
    v5      VARCHAR NOT NULL,
    CONSTRAINT unique_key_sqlx_adapter UNIQUE (ptype, v0, v1, v2, v3, v4, v5)
);
