# Triggers

> [Scylla frontend](../../../../README.md) › `features/` › **triggers** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

A trigger is what starts a pipeline when nobody clicks Run: a schedule, or an inbound webhook.
Triggers belong to a pipeline, and the page lives at
`/:org/projects/:projectId/pipelines/:pipelineId/triggers`, behind `MANAGE_TRIGGERS`.

## Two kinds, one union

The domain models trigger sources as a discriminated union on `kind`:

- **Cron** — a schedule. `CronScheduleBuilder` composes the expression so users are not typing
  `*/15 * * * *` from memory, and `cron.utils.ts` handles parsing and human-readable formatting.
- **Webhook** — an inbound URL. Calling it starts a run. It carries a signing secret and a
  header for the HMAC signature, so Scylla can verify the caller.
- **Unknown** — the forward-compatibility branch.

That third case is worth explaining. When the backend adds a trigger type this frontend has
never heard of, `UnknownSource` is what the mapper produces, and the UI renders "unsupported
trigger type" instead of crashing. Every `switch` over `TriggerKind` must handle it. It is
cheap insurance against a deploy skew that would otherwise blank the page.

Alongside the source, a trigger carries **inputs** — parameters passed to the run, edited
through `TriggerInputsEditor`. Read and write shapes are kept apart: `TriggerEntity` is what you
get back, `TriggerDraft` is what a form produces.

## Operations

| Action | Effect |
|---|---|
| Create / update | define or change the schedule or webhook |
| Enable / disable | pause a trigger, reversibly, keeping its configuration and URL |
| Fire now | run the pipeline immediately, through the trigger |
| Delete | remove it — a webhook's URL is gone for good |

The enable/disable distinction is the one to preserve in the UI. Disabling is a pause; deleting
a webhook invalidates a URL that external systems may be calling, and no amount of recreating
brings the old one back.

**Fire now** is the operation that crosses module boundaries: it creates a job. Its hook
invalidates [jobs](../jobs/README.md)'s query keys as well as its own, which is possible because
`jobs` exports its key factories publicly. Without that, the user would fire a trigger and see
nothing change on the runs page.

## Secrets

A webhook trigger's signing secret is shown once, at creation, and is never retrievable
afterwards — the same reveal-once handling as [apps](../apps/README.md) and
[secret](../secret/README.md). Copy it or lose it; never store or log it.

## Structure

**Domain** holds the entity, the source union and the repository contract.
**Infrastructure** is the standard interface / gRPC impl / mapper trio, with the mapper
responsible for turning an unrecognised proto kind into `Unknown` rather than throwing.
**Presentation** splits into the table (`triggers-table/`, with cells for source and status), the
dialogs (`TriggerFormDialog` composing the cron builder and the inputs editor), and an overview
card summarising what is scheduled.

## Related modules

- [pipeline](../pipeline/README.md) — what a trigger starts.
- [jobs](../jobs/README.md) — the runs it produces.
- [layout](../../layout/README.md) — the shell, and `whats-new.ts` where a release is announced.
