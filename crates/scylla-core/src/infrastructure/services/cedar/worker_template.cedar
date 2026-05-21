// scylla — restricted worker-role template (V1).
//
// Registered under the "worker" role id. Each permission_grants row for a worker
// links an instance binding ?principal to a machine App and ?resource to the
// granted scope (organization / project), conferring only the actions an agent
// needs to pull and execute jobs within that scope: read the pipeline, execute
// its jobs, and report job status / logs. Unlike the full-control role template
// it grants no management, create, or delete actions. Cedar resolves the
// tenancy hierarchy, so a worker scoped to an organization may act on every
// pipeline and job beneath it.
permit (
    principal == ?principal,
    action in [
        Scylla::Action::"readPipeline",
        Scylla::Action::"executeJob",
        Scylla::Action::"writeJobStatus",
        Scylla::Action::"writeJobLog"
    ],
    resource in ?resource
);
