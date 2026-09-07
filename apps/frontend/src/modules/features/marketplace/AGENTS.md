# `features/marketplace` — agent guide

Ready-made pipeline templates a project can start from.

**Layer** `features/` · **id** `marketplace` · **DI key** `marketplaceRepository`

## Import rules

- May import: `@platform/*` barrels, `@shared/*`, other features' `index.ts`.
- Must never import: `core/`, `layout/`, another feature's internals.
- Outside code reaches this module **only** through `index.ts`.

## Public API — `index.ts`

```typescript
type MarketItem
useMarketplace
```

Never add: `marketplace.module.ts`, `use-marketplace-domain.ts`, pages.

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

The domain contract and `useMarketplace` do **not** change — that is the point of the interface.

## Layout

```
marketplace.module.ts                route + nav + DI wiring (private; registry only)
index.ts                             public API
domain/
  structs/market-item.struct.ts      MarketItem
  repository/marketplace.repository.ts
infrastructure/repository/default-marketplace.repository.ts   ← stubbed
presentation/
  hooks/use-marketplace-domain.ts    DI accessor (private)
  hooks/use-marketplace.ts
  stores/use-filter.store.ts         UI filter state (Zustand — correct use)
  ui/Marketplace.page.tsx, MarketplaceTopBar.tsx,
  ui/MarketItemList.tsx, MarketItemCard.tsx
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

- `use-filter.store.ts` is a legitimate Zustand store: it holds **search/filter UI state only**.
  Never put the fetched items in it — the list belongs to TanStack Query.
- Filtering happens during render from the query data plus the store's criteria. No mirror
  state, no effect syncing one into the other.

## Before done

`pnpm typecheck && pnpm lint && pnpm depcruise && pnpm depcruise:cycles` — all clean.
New strings: `pnpm extract && pnpm compile`.
