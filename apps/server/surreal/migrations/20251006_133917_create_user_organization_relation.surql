DEFINE TABLE user_organization TYPE RELATION IN users OUT organizations PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD role ON user_organization TYPE string DEFAULT "member" PERMISSIONS FULL;
DEFINE FIELD joined_at ON user_organization TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

-- ensure a user can only be added to an organization once
DEFINE INDEX unique_user_org ON user_organization FIELDS in, out UNIQUE;

