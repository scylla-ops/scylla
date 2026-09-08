# `shared` — agent guide

Generic UI, hooks and utilities with **no business meaning**.

**Layer** `shared/` (bottom) · aliases `@shared/*`, `@shadcn/*`

## Import rules — the hard one

- **`shared/` MUST NOT import `features/`, `core/`, `layout/` or `platform/`**
  (`shared-is-generic`, error). It is the bottom of the graph and depends on nobody.
- Everyone may import it.
- **Test before adding anything here:** could this live in another product, unchanged? If it
  mentions a pipeline, a job, a role or an organization, it is not shared — it belongs to the
  feature.

## Structure

```
domain/
  structs/pagination.struct.ts       PaginationParams, PaginationInfo
  types/paginated-list.type.ts       PaginatedList<T>
infrastructure/grpc/wrappers.ts      generic proto helpers
utils/                               ← no barrel, import by path
  scylla-result.ts                   ScyllaResult<T>, ScyllaError
  date-utils.ts, slug.ts, status-config.ts, job-status.utils.ts, toast-messages.ts
presentation/
  hooks/                             use-pagination, use-selection, use-dialog,
                                     use-feature-selection, use-resource-error, use-now,
                                     use-code-mirror-theme
  stores/use-selection.store.ts      one of the app's two global stores
  structs/scylla-form.struct.ts      FormItem, FormItemType, FormChange, SelectOption
  ui/index.ts                        re-exports the five groups below
  ui/data-display/                   DataTable, Pagination, ListCard, StatusBar,
                                     CopyableText, status-indicator, AgentRunInstructions
  ui/feedback/                       ErrorState, ConfirmOperationAlertDialog, SecretRevealDialog
  ui/forms/                          ScyllaForm, FormDialog, CheckboxTree
  ui/controls/                       IconButton, BackButton
  ui/layout/                         FeatureHeader, ContextItem, AnimatedOutlet
  ui/shadcn/                         shadcn/ui primitives — @shadcn/*
  utils/                             cn, toast, i18n, code-mirror-theme
locales/                             shared's own catalog
```

`shared` has **no root `index.ts`** — import by path (`@shared/utils/scylla-result.ts`) or from
`ui/index.ts` / a group barrel for components.

## `ScyllaResult<T>` — the error contract

Every async operation returns `ScyllaResult<T>`, never a raw throw.

```typescript
const result = await ScyllaResult.tryAsync(() => api.call(), 'Error message');
result.fold({ onSuccess: data => …, onError: err => … });
const data = result.unwrap();          // throws — do this inside queryFn/mutationFn
```

- Data sources wrap with `tryAsync`; `ScyllaError` extracts the gRPC code.
- Hooks call `.unwrap()` **inside** `queryFn` / `mutationFn` so TanStack Query owns the error.
- `map` / `flatMapAsync` chain without unwrapping (see `UpdateRoleUseCase`).
- **Do not add an `onError` toast in a hook** — `core`'s `QueryCache`/`MutationCache` already
  toasts globally, and you would double it.

## Reuse these — do not reinvent

| Need | Use |
|---|---|
| Row selection | `useSelection(key)` over the single `useSelectionStore` — **no per-feature selection store** |
| List header (count, clear, delete, new) | `FeatureHeader` |
| A form | `FormItem[]` → `ScyllaForm`; `FormDialog` wraps it; `useFormState(items)` owns values/validation — it is exported from `ui/forms/ScyllaForm.tsx`, not from `hooks/` |
| Pagination | `usePagination()` — local page merged with server `totalCount`/`totalPages` |
| A table | `DataTable` (+ `usePagination`) — row keys are business ids, never indices |
| Confirm a destructive action | `ConfirmOperationAlertDialog` |
| Show a one-time secret | `SecretRevealDialog` |
| Error state | `ErrorState` / `useResourceError` |
| Class names | `cn()` |
| Toasts | `toast` from `presentation/utils/toast.ts` |

## Rules that bite here

- **`useSelectionStore` is keyed by feature.** `useSelection('jobs')` and `useSelection('users')`
  are independent. Never add a second selection store.
- It and `useContextStore` (`@platform/context`) are the app's **only** two global stores.
  Everything else is TanStack Query (server state) or local `useState`.
- **Never put server state in a shared store.**
- A component used by ≥ 2 features moves here and gets exported from its group barrel. A
  component used by one stays in that feature.
- `ui/shadcn/` is generated/vendored shadcn/ui. Prefer composing over editing; if you must edit,
  keep the upstream API.
- `status-config.ts` / `job-status.utils.ts` are borderline — they encode status *presentation*
  (colour, icon, label), not business rules. Keep it that way; job semantics belong in
  `features/jobs`.
- Adding to `shared/` needs a second real usage. One usage stays inline.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`. After moving a component between modules:
`node scripts/restore-translations.mjs`.
