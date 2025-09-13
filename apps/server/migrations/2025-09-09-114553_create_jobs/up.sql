CREATE TYPE execution_status AS ENUM (
    'queued',
    'running',
    'succeeded',
    'failed',
    'canceled'
    );

CREATE TABLE jobs
(
    id                   UUID PRIMARY KEY          DEFAULT gen_random_uuid(),
    pipeline_snapshot_id UUID             NOT NULL REFERENCES pipeline_snapshots (id) ON DELETE CASCADE,
    status               execution_status NOT NULL DEFAULT 'queued',
    created_at           TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE stages
(
    id         UUID PRIMARY KEY          DEFAULT gen_random_uuid(),
    job_id     UUID             NOT NULL REFERENCES jobs (id) ON DELETE CASCADE,
    status     execution_status NOT NULL DEFAULT 'queued',
    position   INT              NOT NULL,
    created_at TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE steps
(
    id         UUID PRIMARY KEY          DEFAULT gen_random_uuid(),
    stage_id   UUID             NOT NULL REFERENCES stages (id) ON DELETE CASCADE,
    status     execution_status NOT NULL DEFAULT 'queued',
    position   INT              NOT NULL,
    created_at TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);