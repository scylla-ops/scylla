DEFINE TABLE pipeline_snapshots TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD content ON pipeline_snapshots FLEXIBLE TYPE object READONLY PERMISSIONS FULL;
DEFINE FIELD created_at ON pipeline_snapshots TYPE datetime READONLY VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD pipeline ON pipeline_snapshots TYPE record<pipelines> READONLY PERMISSIONS FULL;