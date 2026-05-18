-- Pipelines store their DAG of nodes as JSONB. We treat the Vec<PipelineNode>
-- as a value object owned by the pipeline aggregate — there's no use case
-- (yet) that filters pipelines by a node attribute, so a normalized
-- `pipeline_nodes` child table would add joins without paying for them.
-- Normalize the day a query like "all pipelines containing a node calling
-- docker" shows up.

CREATE TABLE pipelines (
    id              TEXT        PRIMARY KEY,
    project_id      TEXT        NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    nodes           JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX pipelines_project_id_idx ON pipelines (project_id);
CREATE INDEX pipelines_created_at_idx ON pipelines (created_at DESC);
