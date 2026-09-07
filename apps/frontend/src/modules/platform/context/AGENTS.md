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
useContextStore
useScyllaNavigate
```

Two exports. That is the whole capability.

## The identifier-only rule

`useContextStore` holds **ids and display names, nothing else**:

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

Persistence: Zustand `persist` to `localStorage` under key `scylla-context`, so a reload keeps
the user where they were.

## Cascade on organization change

`setOrganization` **clears `project` and `pipeline`**. A project id from another organization is
meaningless and would produce a broken URL or a cross-tenant query. Preserve that behaviour if
you touch the store.

`reset()` exists for sign-out; the shell also uses `ContextCleaner.wrapper.tsx` to drop context
when leaving a scope.

## Layout

```
index.ts                             public API
use-context.store.ts                 the Zustand store (persisted)
use-scylla-navigate.ts               context-aware navigation
```

No layer folders. Correct — this capability is two files, and inventing
`domain/presentation/` around them would be structure for its own sake.

## Rules that bite here

- **`useScyllaNavigate()` is the only sanctioned way to navigate between scoped screens.** It
  builds URLs from the current context so callers do not hand-assemble
  `/${orgSlug}/projects/${projectId}/…`. Reaching for raw `useNavigate` with a template string
  is how a route change becomes fourteen broken links.
- **The URL is the source of truth, the store is the mirror.** The shell's
  `OrganizationSync.wrapper.tsx` syncs the store *from* route params. Do not add a second sync
  in the other direction — that is the effect cascade this design exists to avoid.
- Server state never goes in here. Only TanStack Query holds fetched data.
- `useContextStore` and `useSelectionStore` (`@shared`) are the app's **only** two global
  stores. Adding a third needs a real justification.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
