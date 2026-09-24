# `platform/context` — agent guide

Which organization / project / pipeline the user is currently looking at, and navigation derived
from it.

**Layer** `platform/` · alias `@platform/context`

## Import rules

- **MUST NEVER import a feature** (`platform-knows-no-feature`, error).
- May import `@shared/*` and other platform capabilities through their `index.ts` only.
- Consumers import `@platform/context` — the barrel, never a deep path.

## Public API — `index.ts`

```typescript
contextStore
scyllaNavigate, type ScyllaNavigate       context-aware navigation
navigateTo, navigateBack                  plain navigation
currentPathname, currentSearch            the current URL
setAppNavigator, type AppNavigator, type NavigateOptions
createResourceError                       NOT_FOUND on a detail page → toast + redirect
```

## The identifier-only rule

`contextStore` holds **ids and display names, nothing else**:

```typescript
organization: { id, name }
project:      { id, name }
pipeline:     { id, name }
setOrganization(id, name)   // also clears project + pipeline
setProject(id, name)
setPipeline(id, name)
reset()
```

This is business context, which is why it is in `platform/` rather than `shared/` — but it is
deliberately identifier-only, so it never needs to know what a project *is*. That is what keeps
it below the features that own those entities.

**Never widen it.** Adding a `ProjectEntity` here would make `platform/` import a feature and
break the layer. If a component needs more than an id and a name, it reads the id from here and
calls that feature's hook.

Persistence: `createStore` (`@shared/presentation/stores/create-store.ts`) keeps it in
`localStorage` under key `scylla-context`, so a reload keeps the user where they were. The stored
format is the format that Zustand used before Phase 6, so a stored context stays valid.

## Cascade on organization change

`setOrganization` **clears `project` and `pipeline`**. A project id from another organization is
meaningless and would produce a broken URL or a cross-tenant query. Preserve that behaviour if
you touch the store.

`reset()` exists for sign-out; the shell also uses `ContextCleaner.wrapper.svelte` to drop
context when leaving a scope.

## Layout

```
index.ts                             public API
context.store.ts                     the store (persisted)
navigator.ts                         setAppNavigator, navigateTo, currentPathname, currentSearch
scylla-navigate.ts                   context-aware navigation
resource-error.svelte.ts             createResourceError
```

## The navigator

`navigator.ts` holds the one navigator of the app. The shell installs it at start-up
(`core.router.ts` calls `setAppNavigator(createAppRouter(…))`), and a test installs a fake one
with `installTestNavigator` from `src/test/navigator.ts`. Nothing else knows the router.

- `navigateTo` accepts a relative target: `'..'` is the parent page, `'members'` is a child
  page.
- `currentPathname()` and `currentSearch()` are reactive while the router is installed.

## Rules that bite here

- **`scyllaNavigate` is the only sanctioned way to navigate between scoped screens.** It builds
  URLs from the current context so callers do not hand-assemble
  `/${orgSlug}/projects/${projectId}/…`. A template string passed to `navigateTo` is how a route
  change becomes fourteen broken links.
- **The URL is the source of truth, the store is the mirror.** The shell's
  `OrganizationSync.wrapper.svelte` syncs the store *from* route params. Do not add a second sync
  in the other direction — that is the effect cascade this design exists to avoid.
- Server state never goes in here. Only TanStack Query holds fetched data.
- `contextStore` and `selectionStore` (`@shared`) are the app's **only** two global
  stores. Adding a third needs a real justification.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
