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
  structs/scylla-form.struct.ts      FormItem, FormItemType, FormValues, SelectOption
  ui/index.ts                        re-exports the five groups below
  ui/data-display/                   DataTable, Pagination, ListCard, StatusBar,
                                     CopyableText, status-indicator, AgentRunInstructions
  ui/feedback/                       ErrorState, ConfirmOperationAlertDialog, SecretRevealDialog
  ui/forms/                          ScyllaForm, FormDialog, CheckboxTree
  ui/controls/                       IconButton, BackButton
  ui/layout/                         FeatureHeader, ContextItem, AnimatedOutlet
  ui/shadcn/                         shadcn/ui primitives — @shadcn/*
  ui/shadcn-svelte/                  their Svelte port, on bits-ui — see below
  ui-svelte/                         the Svelte half of ui/, group for group
  state/                             the Svelte half of hooks/ — runes, not hooks
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
- `getCode()` returns `ScyllaErrorCode`, not `string`: the gRPC-Web status names (derived from
  `GrpcStatusCode`, imported as a type only — nothing lands in the bundle) plus the codes we
  mint. A code compared anywhere must exist in that union, so add yours there first.
- Hooks call `.unwrap()` **inside** `queryFn` / `mutationFn` so TanStack Query owns the error.
- `map` / `flatMapAsync` chain without unwrapping (see `UpdateRoleUseCase`).
- `mapError` rewrites the failure of a result and leaves a success untouched — use it in a data
  source when a generic gRPC code means something more precise for that one call (see
  [`login`](../features/login/AGENTS.md)), rather than special-casing it in every consumer.
- **Do not add an `onError` toast in a hook** — `core`'s `QueryCache`/`MutationCache` already
  toasts globally, and you would double it.

## Reuse these — do not reinvent

| Need | Use |
|---|---|
| Row selection | `useSelection(key)` over the single `useSelectionStore` — **no per-feature selection store** |
| List header (count, clear, delete, new) | `FeatureHeader` |
| A form | `FormItem[]` → `ScyllaForm`; `FormDialog` wraps it; `useFormState(items)` owns values/validation — it is exported from `ui/forms/ScyllaForm.tsx`, not from `hooks/`. Declare the ids in the item type (`readonly FormItem<'name' \| 'description'>[]`) and `onSubmit` receives a typed `FormValues` record — never re-index the values by hand |
| Pagination | `usePagination()` — local page merged with server `totalCount`/`totalPages` |
| How much room the layout left a component | `useMeasuredHeight()` — a `ResizeObserver` behind a callback ref; put it on a container sized by the layout, never by its content |
| A table | `DataTable` (+ `usePagination`) — row keys are business ids, never indices |
| Confirm a destructive action | `ConfirmOperationAlertDialog` |
| Show a one-time secret | `SecretRevealDialog` |
| Error state | `ErrorState` / `useResourceError` |
| Class names | `cn()` |
| Toasts | `toast` from `presentation/utils/toast.ts` |
| Light/dark | `useTheme()` in React; `getTheme`/`setTheme`/`subscribeToTheme` from `stores/theme.store.ts` anywhere else |
| Mounting a Svelte component in the React tree | `SvelteIsland` — see below |
| Reading a Zustand store from Svelte | `toSvelteStore(store, selector)` |
| Translating from Svelte | `t()` from `presentation/utils/i18n-svelte.svelte.ts`, over a `*.messages.ts` |

## The Svelte migration lives partly here

`shared/` holds the bridge, because it is generic and has no business meaning:

```
presentation/ui/svelte/SvelteIsland.tsx    mounts a Svelte component inside React
presentation/stores/to-svelte-store.ts     Zustand store -> Svelte store contract
presentation/stores/theme.store.ts         framework-agnostic; `hooks/use-theme.ts` binds it to React
presentation/utils/i18n-svelte.svelte.ts   `t()` + locale reactivity for components
```

**The fixture proving the platform singletons are reachable is *not* here** — it is
`core/presentation/ui/svelte/`. It imports `@platform/*`, and `shared/` sits below platform and
may not (`shared-is-generic`). Verified: **dependency-cruiser parses `.svelte` and enforces every
rule on it**, so this is caught, not trusted.

The pattern to follow when a store is de-React-ified: the **agnostic core** in
`stores/*.store.ts`, the **React binding** in `hooks/use-*.ts` — the binding is what gets deleted
in Phase 6, the core is what survives. `theme.store.ts` is the worked example.

### `ui-svelte/` and `state/` — the Svelte halves

```
ui-svelte/{controls,data-display,feedback,forms,layout,motion}/   mirrors ui/, group for group
state/                                                           mirrors hooks/
```

Parallel trees rather than `.svelte` files dropped beside their `.tsx` originals: the two versions
of a component share a name, so one barrel cannot export both. `ui/` and `hooks/` are **frozen** —
bug fixes only — and Phase 6 deletes them and renames these into place.

**There are no hooks in `state/`.** `use-selection.ts` became `createSelection(key)`,
`use-pagination.ts` became `createPagination(options)`, `use-feature-selection.ts` became
`createFeatureSelection(key, allIds)`. Three things carry over from the port and will bite again:

- **A list that arrives later is passed as a getter, not an array.** `createFeatureSelection`
  takes `() => string[]` and `createFormState` takes `() => items`. React re-ran the whole hook on
  every render and got this for free; capturing the value once freezes the helper on whatever the
  first, usually empty, render held.
- **`toRune(store)`** (`stores/to-rune.svelte.ts`) is how rune code reads a Zustand store —
  `to-svelte-store.ts` produces the `$store` contract a *template* consumes, which a `.svelte.ts`
  module cannot use. It is built on `createSubscriber`, so the subscription starts only while
  something is reading and the helpers stay testable in plain TypeScript, outside any component.
- **Derive instead of mirroring.** `usePagination` kept `pageSize` in state and ran an effect to
  copy the measured size into it; `createPagination` derives it, so there is no effect and no
  frame where the two disagree. Only "the user picked a size" is remembered, because nothing else
  records it.

DOM measurement is a **Svelte action**, never an effect: `createMeasuredHeight()` returns
`{ height, measure }` and the caller writes `use:measure`. That is also what replaces React's
callback-ref trick for elements that mount late.

`motion/` holds the transitions. `prefersReducedMotion()` is not optional decoration: the
`@media (prefers-reduced-motion: reduce)` block in `index.css` neutralises CSS animations, but a
Svelte transition writes inline styles from JavaScript and that query never sees it — every
transition here asks and collapses its duration to zero.

### `ui/shadcn-svelte/` — the ported primitives

Radix → **`bits-ui`**, with the Tailwind class strings copied verbatim, which is why the design
survives the port untouched. `ui/shadcn/` next door is **frozen** — bug fixes only — so the two
cannot drift while both are alive. Both folders go in Phase 6.

Ported so far, and **only these**: `Button`, `Card` (+ the six parts), `Input`, `Skeleton`,
`Tooltip`, `Dialog`, `AlertDialog`, `Checkbox`, `Avatar`, `Table` (the five parts `DataTable`
composes), `Label`, `Field`/`FieldGroup`/`FieldLabel`, `Select`. A primitive lands here the phase
its first Svelte consumer does — porting the rest now would be components with no usage.

What a port has to get right, all of it invisible to the compiler:

- **`tsc` sees only a `.svelte` file's default export.** Anything a `.ts` must import — a `cva`
  config, a variant type — lives in a `.ts` beside it (`button-variants.ts`), never in
  `<script module>`.
- **`asChild` is bits-ui's `child` snippet**, and it rides through `...rest`. The trigger's props
  land *on* the caller's element: `<TooltipTrigger>{#snippet child({ props })}<Button {...props}/>`.
  The merged `data-slot` wins over the Button's own — `data-variant` is what still identifies it.
- **Parts that carry no styling are aliased from bits-ui in `index.ts`**, not wrapped.
  `Dialog`, `DialogTrigger`, `DialogClose`, `DialogPortal`, `AlertDialog`, `AlertDialogTrigger`.
- **`AlertDialogAction` / `AlertDialogCancel` are plain `Button`s**, not bits-ui's own, which close
  the dialog on click. Every confirmation here keeps the dialog open and disabled while its
  mutation runs; the parent owns `open`. Do not "fix" this.
- The alert dialog is a **real** `role="alertdialog"` that ignores an outside click — something
  `shadcn/alert-dialog.tsx`, built on Radix's plain dialog, never was.
- **bits-ui drops ARIA roles Radix set, and the wrapper puts them back.** The tooltip content had
  no `role="tooltip"`; the select trigger had every piece of the combobox pattern —
  `aria-haspopup`, `aria-expanded`, `aria-activedescendant` — but no `role="combobox"`. Nothing
  breaks loudly: `aria-describedby` still carries the tooltip text, the button still opens. Check
  the role when you port a primitive, and add it to *our* wrapper when it is missing.
- **CSS variable names change**: `--radix-*-content-transform-origin` becomes
  `--bits-floating-transform-origin`, and the select's available-height/anchor-width variables
  likewise. Nothing fails loudly if you miss one.
- **Radix's `Indicator` parts become an `{#if}` in a children snippet** (checkbox, select item),
  because bits-ui hands the state to the snippet instead of mounting a separate node.
- **Highlight styles need `data-highlighted:`, not just `focus:`.** bits-ui never moves DOM focus
  into a listbox; it tracks the active option with `aria-activedescendant`.
- Composed primitives are driven from a `*.fixture.svelte`: their parts are components, so a
  `createRawSnippet` cannot build them.

Three things about testing them, all in the shared harness so nobody re-derives them:

- **`setup.ts` clears `document.body.style.pointerEvents` before every test.** bits-ui locks the
  page behind an open dialog and the lock outlives unmounting, which made the *next* test in the
  file fail with an error pointing at an innocent line.
- **`setup.ts` stubs `Element.prototype.animate`.** jsdom has no Web Animations API, and a Svelte
  `transition:` runs on it. The stub stays *running* rather than resolving, so a leaving node is
  still observable.
- **`findFloating(role, name?)` / `findTooltip()` from `render.svelte.ts`** for anything in a
  floating layer — tooltip content, select options. floating-ui has no layout to measure in jsdom,
  so it leaves the wrapper at `visibility: hidden` forever: `getByRole` skips the subtree, *and*
  the accessible-name algorithm ignores its text, which is why the helper matches `name` against
  the element's text. Do not fall back to `getByText` — that would keep passing if the element
  lost its role, which is exactly the regression above.

An open-outside click is still out of `userEvent`'s reach; use `fireEvent.pointerDown(document.body)`,
which is what the dismiss layer listens for.

In `vite.config.ts`, `vendor-ui-svelte` holds **only Svelte-only packages**. `@floating-ui` and
`tabbable` are shared with Radix: claiming them there moved 8.7 kB gzip of React positioning code
into a Svelte-named chunk and preloaded it from the entry. Leave shared packages unassigned.

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
