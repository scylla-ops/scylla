// scylla — scoped-role template (V1).
//
// One body, registered once per grantable role id (project-admin,
// organization-admin). Each permission_grants row links an instance binding
// ?principal to the user and ?resource to the granted scope, conferring full
// control over that scope and everything beneath it in the entity hierarchy.
permit (
    principal == ?principal,
    action,
    resource in ?resource
);
