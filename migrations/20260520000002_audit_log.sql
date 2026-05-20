-- Persistent authorization audit trail. One row per PermissionService::check
-- decision (allow and deny), written out-of-band by the control-plane.
--
-- `policies` records the Cedar policy ids that determined the verdict (admin
-- rule, ABAC member rule, or a linked grant) for forensics. `principal_id` /
-- `resource_id` are nullable (anonymous principal, System resource).

CREATE TABLE audit_log (
    id             TEXT        PRIMARY KEY,
    occurred_at    TIMESTAMPTZ NOT NULL,
    principal_kind TEXT        NOT NULL,
    principal_id   TEXT,
    action         TEXT        NOT NULL,
    resource_kind  TEXT        NOT NULL,
    resource_id    TEXT,
    decision       TEXT        NOT NULL,
    policies       TEXT[]      NOT NULL DEFAULT '{}',
    reason         TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX audit_log_occurred_at_idx ON audit_log (occurred_at);
CREATE INDEX audit_log_principal_idx   ON audit_log (principal_kind, principal_id);
CREATE INDEX audit_log_resource_idx    ON audit_log (resource_kind, resource_id);
CREATE INDEX audit_log_action_idx      ON audit_log (action);
