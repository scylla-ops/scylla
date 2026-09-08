# `features/triggers` — agent guide

What starts a pipeline without a human: schedules and webhooks.

**Layer** `features/` · **id** `triggers` · **DI key** `triggersRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type TriggerEntity, TriggerDraft, CreatedTrigger
type TriggerSource, TriggerInput
TriggerKind                           (enum — a value, not a type)
usePipelineTriggers, TRIGGERS_QUERY_KEY
```

Never add: `triggers.module.ts`, `use-triggers-domain.ts`, pages.

## Data contract

`TriggersRepository` — `domain/repository/triggers.repository.ts`:

| Method | Returns |
|---|---|
| `listByPipelineId(pipelineId)` | `TriggerEntity[]` |
| `getById(triggerId)` | `TriggerEntity` |
| `create(pipelineId, TriggerDraft)` | `CreatedTrigger` |
| `update(triggerId, TriggerDraft)` | `TriggerEntity` |
| `deleteById(triggerId)` | `void` |
| `setEnabled(triggerId, enabled)` | `TriggerEntity` |
| `fireNow(triggerId)` | `string` — the created job id |

Reach it with `useTriggersDomain()` **inside a hook only**.

## The `TriggerSource` union

`domain/structs/trigger-source.struct.ts` is the heart of the module:

```typescript
TriggerKind = Cron | Webhook | Unknown
TriggerSource = CronSource | WebhookSource | UnknownSource   // discriminated on `kind`
```

- **Always narrow on `kind`**; never read `.cron` off a bare `TriggerSource`.
- **`Unknown` is deliberate forward-compatibility.** A backend that adds a trigger type must not
  crash this UI. Every `switch` needs an `Unknown` branch rendering a graceful "unsupported
  trigger type" — do not `throw`, do not fall through, do not assume exhaustiveness.
- `TriggerDraft` / `TriggerSourceDraft` are the *write* shapes (what a form produces); the
  entity is the *read* shape. Keep them distinct.

## Layout

```
triggers.module.ts                   route + DI wiring (private; registry only)
index.ts                             public API
domain/
  entities/trigger.entity.ts         TriggerEntity, TriggerDraft, CreatedTrigger
  structs/trigger-source.struct.ts   the union above, TriggerInput
  repository/triggers.repository.ts
infrastructure/
  repository/data-sources/triggers-remote.data-source.ts   interface
  data/remote/grpc-triggers-remote.data-source.ts          impl
  repository/mappers/grpc-trigger.mapper.ts
  repository/default-triggers.repository.ts
presentation/
  hooks/use-triggers-domain.ts       DI accessor (private)
  hooks/use-pipeline-triggers.ts     the list query + TRIGGERS_QUERY_KEY
  hooks/use-create-trigger.ts, use-update-trigger.ts, use-delete-trigger.ts,
  hooks/use-set-trigger-enabled.ts, use-fire-trigger-now.ts
  ui/Triggers.page.tsx
  ui/components/TriggersHeader.tsx, TriggersOverview.tsx, index.ts (local barrel)
  ui/dialogs/TriggerFormDialog.tsx, CronScheduleBuilder.tsx, TriggerInputsEditor.tsx
  ui/triggers-table/                 TriggersTable + columns + cells
  utils/cron.utils.ts, trigger-form.utils.ts
```

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `project` | `pipelines/:pipelineId/triggers` | `MANAGE_TRIGGERS` | `TriggersPage` |

**No nav entry** — reached from a pipeline. Breadcrumb highlights `pipelineName`.

One permission covers read and write: seeing a webhook's configuration is close enough to
holding it that they are not split.

## Rules that bite here

- **`fireNow` creates a job.** Invalidate the jobs keys (`JOBS_QUERY_KEY` /
  `JOBS_QUERY_ROOT` from [`features/jobs`](../jobs/AGENTS.md)) as well as `TRIGGERS_QUERY_KEY`,
  or the run the user just started will not appear.
- **Enable/disable ≠ delete.** `setEnabled` is reversible and preserves configuration; deleting
  a webhook trigger invalidates its URL forever. Keep them distinct in the UI.
- Webhook secrets follow the reveal-once rule: shown at creation, never re-fetchable. Never
  store or log one.
- Cron parsing/formatting lives in `cron.utils.ts`; `CronScheduleBuilder` is the only place that
  composes an expression. Do not hand-roll cron strings in a component.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
