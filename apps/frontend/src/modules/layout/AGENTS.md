# `layout` — agent guide

The app shell: sidebar, top bar, breadcrumbs, context selector.

**Layer** `app/` (top, with `core`)

## Import rules

- May import features, platform and shared — but **features only through their `index.ts`**
  (`shell-uses-feature-api`, error).
- **No feature may import `layout/`** (`no-feature-imports-the-shell`, error). If a feature
  seems to need something from here, it belongs in `shared/` or the feature owns it.
- `core`'s router mounts `Layout`; nothing else renders it.

## No `index.ts`

`layout/` has no barrel, because only `core` renders it and it does so by direct path. Do not
add one.

## Layout

```
presentation/structs/nav-section.struct.ts   NavItem, NavSection
presentation/ui/
  Layout.tsx                  the shell — takes `navEntries` from core
  AppSidebar.tsx              sidebar shell
  NavMain.tsx                 renders NavSection[] (collapsible sub-menus)
  NavUser.tsx                 current user + sign-out
  TopBar.tsx                  breadcrumbs + controls
  ScyllaBreadcrumbs.tsx       reads route handles
  ScyllaSidebarTrigger.tsx    collapse toggle
  LanguageSelector.tsx        locale switch
  NewTriggerFeaturePopup.tsx  one-off announcement (see below)
  context-selector/           ContextSelector, CurrentContextDisplay
locales/                      the shell's own catalog
```

## The shell renders, it does not decide

`core` passes `navEntriesFor(modules)` into `Layout`. **The sidebar never hardcodes a link.**
Every entry comes from a module's `nav` declaration, carrying its own `permission` — the same
one its route declares, so a link cannot be visible for a page that will deny you.

Two sections exist: `organization` and `system`. Adding a third means changing `NavSection`
handling here **and** widening `NavEntry['section']` in `@platform/routing`.

`ScyllaBreadcrumbs` reads `handle.breadcrumb` off the matched routes. `Crumb.label` / `detail`
are translated, `highlight` is business data shown verbatim. Nothing here builds a crumb from a
pathname.

## The context selector

`ContextSelector` composes `OrganizationList` and `AddOrganizationDialog`, imported from
[`features/organization`](../features/organization/AGENTS.md)'s **public API** — that is why
those two components are exported from a feature barrel. It writes the choice into
`useContextStore` (`@platform/context`).

Direction matters: `layout` → `organization`, never the reverse.

## Rules that bite here

- **`NewTriggerFeaturePopup` is a one-off announcement**, backed by `useNewFeature(key)` from
  `@shared` (a `localStorage` flag + a window event, so every badge for the same key clears
  together). It is not a notification system. Do not grow one here — and delete the popup once
  the announcement has run its course.
- The shell may read a feature's hook, but must not own business logic. If you are writing
  domain rules in `layout/`, they belong in a feature.
- A component here used by a feature must move to `shared/presentation/ui/` — a feature may not
  import the shell.
- After moving a component in or out of `layout/`, run `node scripts/restore-translations.mjs`
  (Lingui catalogs are per-module; `extract` silently drops the moved strings' French).

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
