DEFINE TABLE pipelines TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD content ON pipelines FLEXIBLE TYPE object PERMISSIONS FULL;
DEFINE FIELD created_at ON pipelines TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
DEFINE FIELD updated_at ON pipelines TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;