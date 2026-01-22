DEFINE TABLE jobs TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD content ON jobs FLEXIBLE TYPE object PERMISSIONS FULL;
DEFINE FIELD pipeline_id ON jobs TYPE record<pipelines> READONLY PERMISSIONS FULL;
DEFINE FIELD status ON jobs TYPE string PERMISSIONS FULL;
DEFINE FIELD created_at ON jobs TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
DEFINE FIELD updated_at ON jobs TYPE datetime VALUE time::now() PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

DEFINE INDEX unique_ulid_in_content ON jobs FIELDS content.ulid UNIQUE;