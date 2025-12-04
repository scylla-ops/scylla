DEFINE TABLE jobs TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD content ON jobs FLEXIBLE TYPE object PERMISSIONS FULL;
DEFINE FIELD created_at ON jobs TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
DEFINE FIELD pipeline ON jobs TYPE record<pipelines> READONLY PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

DEFINE INDEX unique_ulid_in_content ON jobs FIELDS content.ulid UNIQUE;