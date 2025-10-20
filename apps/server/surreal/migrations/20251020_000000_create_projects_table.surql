DEFINE TABLE projects TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD name ON projects TYPE string PERMISSIONS FULL;
DEFINE FIELD description ON projects TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD organization ON projects TYPE record<organizations> PERMISSIONS FULL;
DEFINE FIELD is_active ON projects TYPE bool DEFAULT true PERMISSIONS FULL;
DEFINE FIELD created_at ON projects TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
DEFINE FIELD updated_at ON projects TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

-- ensure project names are unique within an organization
DEFINE INDEX unique_name_per_org ON projects FIELDS organization, name UNIQUE;


