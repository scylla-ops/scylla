# `shared` — agent guide

Generic UI, state helpers and utilities with **no business meaning**.

**Layer** `shared/` (bottom) · aliases `@shared/*`, `@shadcn`

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
  ui/index.ts                        `@shared/presentation/ui` — re-exports the groups below
  ui/controls/                       IconButton, GatedButton
  ui/data-display/                   DataTable, Pagination, PaginationSlot, StatusBar,
                                     StatusIndicator, CopyableText, TruncatedText,
                                     AgentRunInstructions
  ui/feedback/                       ScyllaDialog, ErrorState, ConfirmOperationAlertDialog, SecretRevealDialog
  ui/forms/                          ScyllaForm, FormDialog, createFormState, FormItem types
  ui/layout/                         FeatureHeader, ContextItem, PageTransition,
                                     ScyllaLoadingScreen
  ui/motion/                         page transitions, reduced motion
  ui/editor/code-mirror.actions.ts   renderCodeMirror (a Svelte action)
  ui/icon.ts                         LucideIcon (the prop type of an icon)
  ui/shadcn/                         shadcn-svelte primitives on bits-ui — `@shadcn`
  state/                             createSelection, createPagination, createFeatureSelection,
                                     createMeasuredHeight, createCompactContainer, createNow
  stores/                            createStore, toRune, theme.store.ts, selection.store.ts
  utils/                             cn, toast, i18n, i18n-svelte, code-mirror-theme
locales/                             shared's own catalog
```

`shared` has **no root `index.ts`** — import by path (`@shared/utils/scylla-result.ts`) or from
`@shared/presentation/ui` / a group barrel for components.

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
- Query and mutation options call `.unwrap()` **inside** `queryFn` / `mutationFn` so TanStack Query owns the error.
- `map` / `flatMapAsync` chain without unwrapping (see `UpdateRoleUseCase`).
- `mapError` rewrites the failure of a result and leaves a success untouched — use it in a data
  source when a generic gRPC code means something more precise for that one call (see
  [`login`](../features/login/AGENTS.md)), rather than special-casing it in every consumer.
- **Do not add an `onError` toast to a query or a mutation** — the `QueryCache`/`MutationCache`
  of `@platform/query` already toasts globally, and you would double it.

## Reuse these — do not reinvent

| Need | Use |
|---|---|
| Row selection | `createSelection(key)` over the single `selectionStore` — **no per-feature selection store** |
| Selection for a list header | `createFeatureSelection(key, () => ids, { deleteItem })` → `headerProps` for `FeatureHeader` |
| List header (count, clear, delete, new) | `FeatureHeader` |
| A form | `FormItem[]` → `ScyllaForm`; `FormDialog` wraps it; `createFormState(() => items)` owns values/validation. Declare the ids in the item type (`readonly FormItem<'name' \| 'description'>[]`) and `onSubmit` receives a typed `FormValues` record — never re-index the values by hand |
| Pagination | `createPagination(options)` — local page merged with server `totalCount`/`totalPages`; `responsive: true` sizes the page to the container |
| How much room the layout left a component | `createMeasuredHeight()` → `{ height, measure }`, and `use:measure` on a container sized by the layout, never by its content |
| A table | `DataTable` (+ `createPagination`) — row keys are business ids, never indices |
| An action the user may not use | `GatedButton` with `allowed={can(…)}` |
| A modal | `ScyllaDialog` (`open`, `onOpenChange`, `title`, `dismissible`). Never `Dialog` + `DialogContent` directly: `ScyllaDialog` rebuilds its content at each opening without recreating `DialogContent`, which is what keeps it closable |
| Confirm a destructive action | `ConfirmOperationAlertDialog` |
| Show a one-time secret | `SecretRevealDialog` |
| Error state | `ErrorState`; `createResourceError` (`@platform/context`) on a detail page |
| Class names | `cn()` |
| Toasts | `toast` from `presentation/utils/toast.ts` (`svelte-sonner`) |
| Light/dark | `getTheme` / `setTheme` / `currentTheme()` (reactive) from `stores/theme.store.ts` |
| A store | `createStore` from `stores/create-store.ts`; rune code reads it with `toRune(store)` |
| Translating | `t()` from `presentation/utils/i18n-svelte.svelte.ts`, over a `*.messages.ts` |
| A page transition | `PageTransition` — `@platform/routing` already wraps every page in it |

## Rune helpers: `state/`

There are no hooks. Three rules carry over from the migration and still bite:

- **A list that arrives later is passed as a getter, not an array.** `createFeatureSelection`
  takes `() => string[]` and `createFormState` takes `() => items`. Capturing the value once
  freezes the helper on whatever the first, usually empty, render held.
- **`toRune(store)`** (`stores/to-rune.svelte.ts`) is how rune code reads a store. It is built on
  `createSubscriber`, so the subscription starts only while something is reading and the helpers
  stay testable in plain TypeScript, outside any component.
- **Derive instead of mirroring.** `createPagination` derives its page size from the measured
  height, so there is no effect and no frame where the two disagree. Only "the user picked a
  size" is remembered, because nothing else records it.

DOM measurement is a **Svelte action**, never an effect: `createMeasuredHeight()` returns
`{ height, measure }` and the caller writes `use:measure`.

`ui/motion/` holds the transitions. `prefersReducedMotion()` is not optional decoration: the
`@media (prefers-reduced-motion: reduce)` block in `index.css` neutralises CSS animations, but a
Svelte transition writes inline styles from JavaScript and that query never sees it — every
transition here asks and collapses its duration to zero.

### `ui/shadcn/` — the primitives

The shadcn-svelte primitives, on **`bits-ui`**, with the Tailwind class strings of the old
shadcn/ui components copied verbatim. Import them from `@shadcn`. A primitive is ported when its
first consumer needs it — there is no primitive here without a consumer.

The sidebar (`Sidebar*`, `setSidebar` / `getSidebar`) keeps its open state in a Svelte context
that `SidebarProvider` creates. `sidebar-state.svelte.ts` holds it; the mobile layout uses the
`svelte/reactivity` `MediaQuery`.

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
- The alert dialog is a **real** `role="alertdialog"` that ignores an outside click.
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

In `vite.config.ts`, `vendor-ui` holds bits-ui and everything it pulls (`@floating-ui`,
`tabbable`, `runed`, `svelte-toolbelt`), lucide and `svelte-sonner`. `clsx` goes in
`vendor-svelte`, because the Svelte runtime imports it too: in `vendor-ui` it made a circular
chunk.

## Rules that bite here

- **`selectionStore` is keyed by feature.** `createSelection('jobs')` and
  `createSelection('users')` are independent. Never add a second selection store.
- It and `contextStore` (`@platform/context`) are the app's **only** two global UI stores
  (`permissionsStore` in `@platform/authz` holds the permissions). Everything else is TanStack
  Query (server state) or local `$state`.
- **Never put server state in a shared store.**
- A component used by ≥ 2 features moves here and gets exported from its group barrel. A
  component used by one stays in that feature.
- `ui/shadcn/` is ported shadcn-svelte. Prefer composing over editing; if you must edit, keep the
  upstream API.
- `status-config.ts` / `job-status.utils.ts` are borderline — they encode status *presentation*
  (colour, icon, label), not business rules. Keep it that way; job semantics belong in
  `features/jobs`.
- Adding to `shared/` needs a second real usage. One usage stays inline.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`. After moving a component between modules:
`node scripts/restore-translations.mjs`.
