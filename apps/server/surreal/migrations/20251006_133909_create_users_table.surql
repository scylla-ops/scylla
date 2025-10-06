DEFINE TABLE users TYPE ANY SCHEMAFULL PERMISSIONS NONE;

-- ------------------------------
-- FIELDS
-- ------------------------------

DEFINE FIELD created_at ON users TYPE datetime DEFAULT time::now() READONLY PERMISSIONS FULL;
DEFINE FIELD is_active ON users TYPE bool DEFAULT false PERMISSIONS FULL;
DEFINE FIELD password_hash ON users TYPE string PERMISSIONS FULL;
DEFINE FIELD updated_at ON users TYPE datetime VALUE time::now() PERMISSIONS FULL;
DEFINE FIELD username ON users TYPE string PERMISSIONS FULL;

-- ------------------------------
-- INDEXES
-- ------------------------------

DEFINE INDEX unique_username_ci ON users FIELDS username UNIQUE;