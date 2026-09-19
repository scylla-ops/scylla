# `features/marketplace` — agent guide

Ready-made pipeline templates a project can start from.

**Layer** `features/` · **id** `marketplace` · **DI key** `marketplaceRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

**Presentation is Svelte** (Phase 2 of `refacto_svelte.md`). Domain and infrastructure are
unchanged. There is no `use-<feature>-domain.ts` and no hooks: reads and writes are declared as
options objects in `presentation/*.queries.ts`, which both bindings can run — `createQuery` here,
react-query's `useQuery` in the modules still on React.

## Public API — `index.ts`

```typescript
type MarketItem
marketplaceQueries, MARKETPLACE_QUERY_KEY
```

Never add: `marketplace.module.ts`, pages.

## Data contract

`MarketplaceRepository` — `domain/repository/marketplace.repository.ts` (a **default** export):

| Method | Returns |
|---|---|
| `getMarketplace()` | `ScyllaResult<MarketItem[]>` |

## ⚠ The repository is stubbed

`DefaultMarketplaceRepository` returns **hardcoded in-memory items**. It has no data source, no
mapper, and carries a `TODO` saying so. That is why `marketplace.module.ts` constructs it with
`new DefaultMarketplaceRepository()` and no `grpcTransport` — the only module in the codebase
that takes no transport.

When wiring the real backend:

1. Add `infrastructure/repository/data-sources/marketplace-remote.data-source.ts` (interface).
2. Add `infrastructure/data/remote/grpc-marketplace-remote.data-source.ts` (impl).
3. Add `infrastructure/repository/mappers/grpc-market-item.mapper.ts`.
4. Inject the data source into `DefaultMarketplaceRepository` and pass `grpcTransport` in
   `marketplace.module.ts`.
5. Delete the `TODO` and the inline fixtures.

The domain contract and `marketplaceQueries` do **not** change — that is the point of the
interface.

## Layout

```
marketplace.module.ts                route + nav + DI wiring (private; registry only)
index.ts                             public API
domain/
  structs/market-item.struct.ts      MarketItem
  repository/marketplace.repository.ts
infrastructure/repository/default-marketplace.repository.ts   ← stubbed
presentation/
  marketplace.queries.ts                    the catalog read
  marketplace-filter.state.svelte.ts        search box state + `matchesFilter`
  ui/Marketplace.page.svelte, MarketplaceTopBar.svelte,
  ui/MarketItemList.svelte, MarketItemCard.svelte
  ui/marketplace.messages.ts
```

`MarketItem` is a **struct**, not an entity: a catalog listing with no identity the app owns.

## Routes & nav

| Mount | Path | Permission | Component |
|---|---|---|---|
| `organization` | `marketplace` | `LIST_APPS_BY_ORGANIZATION` | `MarketplacePage` |

Sidebar: section `organization`, order `50`, icon `ShoppingCartIcon`.

The permission is a stand-in — the browsing gate was borrowed from apps and should be revisited
when the real catalog lands.

## Rules that bite here

- `marketplace-filter.state.svelte.ts` is module-level `$state` — the rune equivalent of the
  Zustand store it replaced, and still **search/filter UI state only**. Never put the fetched
  items in it; the list belongs to TanStack Query.
- **`MarketplaceTopBar`, `MarketItemList` and `MarketItemCard` are not mounted by any page.**
  `Marketplace.page.svelte` is still the "available soon" placeholder. They were ported rather
  than dropped because they are the screen this page becomes once the repository is real.
- Filtering happens during render from the query data plus the store's criteria. No mirror
  state, no effect syncing one into the other.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`
— all clean.
New strings: `pnpm extract && pnpm compile`.
