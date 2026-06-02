-- The `permission_` prefix was redundant noise: a row here is a grant, full
-- stop. Rename the table to `grants`. Indexes and constraints follow the rename
-- automatically (their names still reference the old prefix but remain valid).
ALTER TABLE permission_grants RENAME TO grants;
