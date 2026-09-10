# Shared

> [Scylla frontend](../../../README.md) › `shared/` ·
> [agent guide](./AGENTS.md) · [architecture](../../../docs/architecture.md)

The bottom layer: generic components, hooks and utilities with **no business meaning**. Tables,
forms, dialogs, pagination, the Result type, the shadcn/ui primitives.

Everything may import `shared`. `shared` imports nobody — not features, not the shell, not even
`platform/`. It is the only module in the codebase with that property, and dependency-cruiser
enforces it.

## The test for what belongs here

> Could this live in a completely different product, unchanged?

If the answer is yes, it belongs in `shared`. If it mentions a pipeline, a job, a role or an
organization — even in a type name — it belongs to the feature that owns that concept.

The rule is not aesthetic. `shared` sits below `platform/`, which sits below `features/`. A
business concept leaking down here would either drag a feature dependency with it, inverting the
layering, or duplicate a concept the feature already owns.

The corollary is a rule about *timing*: something moves here at its **second real usage**, not
in anticipation of one. A helper used once stays inline in the feature that uses it.

## `ScyllaResult<T>` — errors as values

Async operations in this codebase do not throw. They return `ScyllaResult<T>`:

```typescript
const result = await ScyllaResult.tryAsync(() => api.call(), 'Could not load secrets');

result.fold({
  onSuccess: data => …,
  onError: error => …,
});
```

Data sources wrap their calls in `tryAsync`, so a gRPC failure becomes a `ScyllaError` carrying
the extracted status code and a user-facing message, travelling up as a value that cannot be
forgotten in the way a `catch` can.

At the presentation boundary the convention flips: query and mutation hooks call `.unwrap()`
*inside* `queryFn` / `mutationFn`, which throws — deliberately — so TanStack Query owns loading,
error and retry state from there on. And because [core](../core/README.md) installs a global
error handler on both the query and mutation caches, a feature hook should **not** add its own
error toast. The root already shows one.

For chaining without unwrapping, `map` and `flatMapAsync` compose results — that is how
`UpdateRoleUseCase` reads a role, transforms it and saves it in one expression.

## Patterns worth knowing before you build

Most screens in Scylla are a list, a header and a form, and the shared layer is why they look
and behave alike:

- **`DataTable`** with **`usePagination()`** — local page state merged with the server's
  `totalCount` and `totalPages`. Row keys are business ids, never array indices.
- **`useSelection(key)`** over a single `useSelectionStore`. Selection is keyed by feature, so
  `useSelection('jobs')` and `useSelection('users')` are independent — which is why there is no
  per-feature selection store anywhere in the codebase.
- **`FeatureHeader`** — the list header: item count, clear/delete selection, new-item button.
- **`ScyllaForm`** — forms are declared as `FormItem[]` rather than assembled by hand.
  `FormDialog` wraps one in a dialog, `useFormState` owns values, dirty-checking and validation.
  Both are generic over the item ids, so a form declared with literal ids submits a typed
  `FormValues` record (`{ name: string; description: string }`) instead of a bag of pairs the
  caller has to search through.
- **`ConfirmOperationAlertDialog`** for destructive actions, **`SecretRevealDialog`** for values
  shown exactly once.
- **`CheckboxTree`** — used by the role editor's permission tree.

## Two global stores, and only two

`useSelectionStore` lives here; `useContextStore` lives in
[platform/context](../platform/context/README.md). Those are the *only* application-wide stores.

Everything else is either server state — which belongs in TanStack Query, never in Zustand — or
local component state. A feature may add a store for genuinely ephemeral UI state (the pipeline
editor's draft script, the marketplace's filter criteria), but it stays inside that feature and
never holds fetched data.

## Layout

- `domain/` — `PaginatedList<T>` and the pagination structs. Generic containers, no business
  types.
- `infrastructure/grpc/` — small generic proto helpers.
- `utils/` — `ScyllaResult`, dates, slugs, status presentation, toast messages.
- `presentation/ui/` — components in five groups (`data-display`, `feedback`, `forms`,
  `controls`, `layout`), each with a barrel, plus `shadcn/` for the vendored primitives.

There is no root `index.ts`; import from a group barrel or by path. `shadcn/` is generated
vendor code — compose around it rather than editing it, so upstream updates stay applicable.

One borderline case, called out honestly: `status-config.ts` and `job-status.utils.ts` encode
how a status is *presented* — its colour, icon and label. That is generic. What a status
*means* — whether a job is running or finished — lives in
[features/jobs](../features/jobs/README.md) as domain logic. Keep the line there.

## Related modules

- [core](../core/README.md) — global error handling built on `ScyllaResult`.
- [layout](../layout/README.md) — the shell, built from these primitives.
- [platform/context](../platform/context/README.md) — the other global store.
