DEFINE TABLE jobs TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD content ON jobs FLEXIBLE TYPE object PERMISSIONS FULL;
DEFINE FIELD created_at ON jobs TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
DEFINE FIELD snapshot ON jobs TYPE record<pipeline_snapshots> READONLY PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

DEFINE INDEX unique_uuid_in_content ON jobs FIELDS content.uuid UNIQUE;