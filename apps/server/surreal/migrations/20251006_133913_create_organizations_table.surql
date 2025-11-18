DEFINE TABLE organizations TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD name ON organizations TYPE string PERMISSIONS FULL;
DEFINE FIELD description ON organizations TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD is_active ON organizations TYPE bool DEFAULT true PERMISSIONS FULL;
DEFINE FIELD created_at ON organizations TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
DEFINE FIELD updated_at ON organizations TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

-- ensure organization names are unique
DEFINE INDEX unique_name ON organizations FIELDS name UNIQUE;
