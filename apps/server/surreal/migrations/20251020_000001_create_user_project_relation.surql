DEFINE TABLE user_project TYPE RELATION IN users OUT projects PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD role ON user_project TYPE string DEFAULT "member" PERMISSIONS FULL;
DEFINE FIELD joined_at ON user_project TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

-- ensure a user can only be added to a project once
DEFINE INDEX unique_user_project ON user_project FIELDS in, out UNIQUE;
