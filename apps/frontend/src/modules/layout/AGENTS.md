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
whats-new.ts                           THE release announcement (see below)
presentation/
  structs/nav-section.struct.ts        NavItem, NavSection
  whats-new.svelte.ts                  seen flags (reactive), `highlightIdForNav`
  nav-sections.ts                      entries → sidebar sections (pure, tested)
  breadcrumbs.ts                       route trail → breadcrumb items (pure, tested)
  shell.state.svelte.ts                organizations query, permission sync, first organization
  sign-out.ts
  ui/
    Layout.svelte                      the shell — takes `navEntries` and `children` from core
    AppSidebar.svelte                  sidebar shell
    NavMain.svelte                     renders NavSection[]
    NavUser.svelte                     current user + sign-out
    TopBar.svelte                      breadcrumbs + controls
    ScyllaBreadcrumbs.svelte           reads the route trail
    ScyllaSidebarTrigger.svelte        collapse toggle
    LanguageSelector.svelte            locale switch
    WhatsNewDialog.svelte              first-launch release announcement
    NewBadge.svelte                    "New" pill on a sidebar entry
    FirstOrganization.svelte           welcome screen when the user has no organization
    layout.messages.ts                 every message of this module (lingui does not read .svelte)
    context-selector/                  OrganizationSelector, CurrentContextDisplay
locales/                               the shell's own catalog
```

## The shell renders, it does not decide

`core` passes `navEntriesFor(modules)` into `Layout`. **The sidebar never hardcodes a link.**
Every entry comes from a module's `nav` declaration, carrying its own `permission` — the same
one its route declares, so a link cannot be visible for a page that will deny you.

Two sections exist: `organization` and `system`. Adding a third means changing `NavSection`
handling here **and** widening `NavEntry['section']` in `@platform/routing`.

`ScyllaBreadcrumbs` reads `handle.breadcrumb` off the route trail (`routeTrail()` from
`@platform/routing`). `Crumb.label` / `detail`
are translated, `highlight` is business data shown verbatim. Nothing here builds a crumb from a
pathname.

## The context selector

`OrganizationSelector` is a dropdown menu. It shows `OrganizationList` from
[`features/organization`](../features/organization/AGENTS.md), and it gives the list
`DropdownMenuItem` as the row component, so each row gets the keyboard focus of the menu. It
opens `AddOrganizationDialog` from the same module. Both come through the **loaders** of the
barrel (`loadOrganizationList`, `loadAddOrganizationDialog`): a barrel that the shell imports
never exports a component.

Direction matters: `layout` → `organization`, never the reverse.

## The shell state

`createShellState()` (`shell.state.svelte.ts`) is created once, in `Layout`. It loads the
organizations of the user, and an effect calls `syncMyPermissions` (`features/roles`) when the
active organization or project changes. It is the only writer of the permissions store.

## Rules that bite here

- **Announcing a new feature is editing `whats-new.ts`, nothing else.** Add a highlight
  (`id`, `title`, `description`, `icon`, and `navUrl` when it is a page) and the first-launch
  dialog and the sidebar badge follow. Never hand-roll a popup or a badge in a feature again —
  that is what this replaced. Bump `WHATS_NEW.version` and replace the highlights at each
  release: the `localStorage` seen flags are keyed by the version, so the announcement re-arms
  by itself. It is not a notification system; do not grow one here.
- The shell may read a feature's hook, but must not own business logic. If you are writing
  domain rules in `layout/`, they belong in a feature.
- A component here used by a feature must move to `shared/presentation/ui/` — a feature may not
  import the shell.
- Page transitions are not here. `RoutePage` in `@platform/routing` animates the page, and
  `Layout` renders `children` directly.
- After moving a component in or out of `layout/`, run `node scripts/restore-translations.mjs`
  (Lingui catalogs are per-module; `extract` silently drops the moved strings' French).

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
