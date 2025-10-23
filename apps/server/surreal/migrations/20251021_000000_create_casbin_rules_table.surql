-- migration to create casbin_rules table for storing RBAC policies
DEFINE TABLE casbin_rules SCHEMAFULL;

DEFINE FIELD ptype ON TABLE casbin_rules TYPE string;
DEFINE FIELD v0 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v1 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v2 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v3 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v4 ON TABLE casbin_rules TYPE option<string>;
DEFINE FIELD v5 ON TABLE casbin_rules TYPE option<string>;

-- index for efficient policy lookups
DEFINE INDEX casbin_ptype_idx ON TABLE casbin_rules COLUMNS ptype;

