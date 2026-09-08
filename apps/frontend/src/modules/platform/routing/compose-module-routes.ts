import type { RouteObject } from 'react-router-dom';
import type { ModuleRoute, NavEntry, RouteMount, ScyllaModule } from './scylla-module.struct.ts';

/**
 * Every sidebar entry the modules declared, ordered.
 *
 * Same declarations the routes come from, so a link and the page it points at
 * can no longer disagree about which permission they need.
 */
export const navEntriesFor = (modules: readonly ScyllaModule[]): NavEntry[] =>
  modules
    .flatMap(module => module.nav ?? [])
    .slice()
    .sort((a, b) => (a.order ?? 0) - (b.order ?? 0));

/** A module route flattened into what react-router actually wants. */
const toRouteObject = (route: ModuleRoute): RouteObject => {
  const { mount: _mount, permission, breadcrumb, children, ...rest } = route;

  // `handle` is static metadata, so it works alongside `lazy`: the guard and the
  // breadcrumbs can read it without waiting for the chunk to load.
  const hasHandle = permission !== undefined || breadcrumb !== undefined;

  return {
    ...rest,
    ...(hasHandle ? { handle: { permission, breadcrumb } } : {}),
    ...(children ? { children: children.map(toRouteObject) } : {}),
  } as RouteObject;
};

/**
 * Folds sibling routes that claim the same path segment into one.
 *
 * Two modules can legitimately own different pages under the same scope —
 * `user` owns `users` (the directory) while `organization` owns `users/:userId`
 * (the settings page, because it renders the organizations panel). Merging lets
 * each declare its own part without either importing the other, and keeps the
 * shared ancestor's breadcrumb on both.
 */
const mergeSharedParents = (routes: RouteObject[]): RouteObject[] => {
  const merged: RouteObject[] = [];
  const byPath = new Map<string, RouteObject>();

  for (const route of routes) {
    const existing = route.path === undefined ? undefined : byPath.get(route.path);

    if (!existing) {
      // Cast: spreading a RouteObject loses the index/non-index discrimination
      // that the union encodes, and `toRouteObject` already preserved it.
      const copy = {
        ...route,
        ...(route.children ? { children: [...route.children] } : {}),
      } as RouteObject;
      if (route.path !== undefined) byPath.set(route.path, copy);
      merged.push(copy);
      continue;
    }

    existing.children = [...(existing.children ?? []), ...(route.children ?? [])];
    // Whichever contributor declared a handle keeps it; defined values win so
    // the merge does not depend on registry order.
    existing.handle = { ...(route.handle ?? {}), ...(existing.handle ?? {}) };
  }

  return merged;
};

/**
 * Collects the routes every module declared for one mount point.
 *
 * Modules are visited in registry order, so two modules contributing to the same
 * scope stay in a predictable order.
 */
export const routesFor = (modules: readonly ScyllaModule[], mount: RouteMount): RouteObject[] =>
  mergeSharedParents(
    modules.flatMap(module =>
      (module.routes ?? []).filter(route => route.mount === mount).map(toRouteObject),
    ),
  );
