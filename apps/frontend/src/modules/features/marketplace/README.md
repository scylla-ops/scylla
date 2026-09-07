# Marketplace

> [Scylla frontend](../../../../README.md) › `features/` › **marketplace** ·
> [agent guide](./AGENTS.md) · [architecture](../../../../docs/architecture.md)

A catalog of ready-made pipeline templates, so a new project does not start from a blank canvas.
The page lists items, lets you search and filter them, and shows what each one does.

Reachable at `/:org/marketplace`, with a sidebar entry under the organization section.

## Status: front-end complete, backend pending

The UI works. The data does not come from anywhere yet.

`DefaultMarketplaceRepository` returns a hardcoded list of items with a `TODO` on it. It is the
only repository in the codebase built without a data source, and `marketplace.module.ts` is the
only module declaration that does not take `grpcTransport` — because there is nothing to
transport.

This is a reasonable state to be in, and it illustrates why the layering pays off. The page,
the hook and the domain contract are all written against `MarketplaceRepository.getMarketplace()`.
When the backend endpoint appears, the work is confined to `infrastructure/`: add a data source
interface and its gRPC implementation, add a mapper, inject the data source into the repository,
pass the transport in the module file. `MarketItem`, `useMarketplace` and every component stay
exactly as they are.

## How it is built

**Domain** is one struct and one interface. `MarketItem` is a *struct* rather than an entity
because a catalog listing has no identity the application owns — it is a description of
something you can copy, not a thing you can mutate. `MarketplaceRepository` is exported as a
default export, unlike its siblings; that is a leftover from an earlier convention rather than a
statement.

**Presentation** splits into the page, a top bar carrying search and filters, a list and a card.
Filter state lives in `use-filter.store.ts`, a Zustand store — and this is a textbook correct
use of one: it holds the user's *criteria*, not the data. The items themselves stay in TanStack
Query, and the visible list is derived during render by applying the criteria to the query
result. Nothing is mirrored, nothing is synced by an effect.

## Related modules

- [pipeline](../pipeline/README.md) — where a chosen template ends up.
- [project](../project/README.md) — the scope a template is instantiated into.
- [apps](../apps/README.md) — where the current (placeholder) permission gate comes from.
