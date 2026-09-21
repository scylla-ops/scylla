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
triggerQueries, triggerMutations, TRIGGERS_QUERY_KEY
```

**This module is Svelte** (Phase 3). There is no `use-triggers-domain.ts` and no hooks: the
factories in `presentation/triggers.queries.ts` resolve the repository through
`getModuleDomain('triggers')` per call, and a component runs them with `createQuery` /
`createMutation` from `@platform/query`.

Never add: `triggers.module.ts`, pages, and never a `.svelte` component — a barrel that exports
one cannot be tree-shaken out of whoever imports it for a type.

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

All wrapped in `ScyllaResult`. Reach it from `triggers.queries.ts` only — never from a component.

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
  triggers.queries.ts                every read and write, as query/mutation options
  ui/Triggers.page.svelte
  ui/triggers.messages.ts            every string — `lingui extract` cannot read `.svelte`
  ui/components/TriggersHeader.svelte, TriggersOverview.svelte, index.ts (local barrel)
  ui/dialogs/TriggerFormDialog.svelte   the shell: `{#key open}` is the form's reset
  ui/dialogs/TriggerForm.svelte         the fields, so recreating it re-seeds them
  ui/dialogs/CronScheduleBuilder.svelte, TriggerInputsEditor.svelte
  ui/triggers-table/                 TriggersTable + trigger-columns.ts + cells
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

- **`fireNow` creates a job**, so `triggerMutations.fireNow` invalidates the pipeline's jobs list
  as well as `TRIGGERS_QUERY_KEY` — otherwise the run the user just started appears nowhere. That
  key is **spelled out locally**, not imported from [`features/jobs`](../jobs/AGENTS.md): one
  array is not worth a module edge, and the graph is cheaper kept acyclic. `jobs`' own
  `JOBS_QUERY_KEY` is the authority — if it changes, this must follow.
- **Do not re-invalidate at the call site.** `triggerMutations` already does it in `onSuccess`.
- **`setEnabled` is optimistic**, with a rollback in `onError`: the switch has to move under the
  pointer, and a round trip's worth of "nothing happened" on a toggle reads as a broken control.
  Keep the rollback honest if you touch it.
- **`onCreated?.(await mutateAsync(draft))` is a trap** — optional-call short-circuits, so with no
  callback the argument is never evaluated: nothing is created and the dialog still closes.
  `TriggerForm` awaits into a binding first. Do not fold it back.
- **Enable/disable ≠ delete.** `setEnabled` is reversible and preserves configuration; deleting
  a webhook trigger invalidates its URL forever. Keep them distinct in the UI.
- Webhook secrets follow the reveal-once rule: shown at creation, never re-fetchable. Never
  store or log one.
- Cron parsing/formatting lives in `cron.utils.ts`; `CronScheduleBuilder` is the only place that
  composes an expression. Do not hand-roll cron strings in a component.
- **`CronScheduleBuilder` does not emit on mount**, deliberately: its parent seeds its own state
  from the same string, so a mount-time emit only re-sends what the parent already has. The React
  effect did, on every open.
- **A dialog's form lives in its own component**, and `{#key open}` in the parent is what resets
  it — recreating the child re-runs its `$state` initializers. That is the reset React needed an
  effect on `[open, trigger]` for. No effect watching `open`.
- **New strings go in `ui/triggers.messages.ts`, never inside a `.svelte`.** Extraction does not
  read components, so a message declared there vanishes from the catalogs without failing a gate.
- **A bits-ui select or menu lives in a floating layer**, which floating-ui leaves
  `visibility: hidden` under jsdom: `getByRole('option')` finds nothing. Use `findFloating` from
  `src/test/render.svelte.ts`.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
