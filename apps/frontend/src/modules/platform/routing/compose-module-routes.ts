import type { AppRoute } from './app-route.struct.ts';
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

const toAppRoute = (route: ModuleRoute): AppRoute => {
  const { mount: _mount, permission, breadcrumb, children, ...rest } = route;
  const hasHandle = permission !== undefined || breadcrumb !== undefined;

  return {
    ...rest,
    ...(hasHandle ? { handle: { permission, breadcrumb } } : {}),
    ...(children ? { children: children.map(toAppRoute) } : {}),
  };
};

const mergeSharedParents = (routes: AppRoute[]): AppRoute[] => {
  const merged: AppRoute[] = [];
  const byPath = new Map<string, AppRoute>();

  for (const route of routes) {
    const existing = route.path === undefined ? undefined : byPath.get(route.path);

    if (!existing) {
      const copy = { ...route, ...(route.children ? { children: [...route.children] } : {}) };
      if (route.path !== undefined) byPath.set(route.path, copy);
      merged.push(copy);
      continue;
    }

    existing.children = [...(existing.children ?? []), ...(route.children ?? [])];
    existing.handle = { ...(route.handle ?? {}), ...(existing.handle ?? {}) };
  }

  return merged;
};

/**
 * Collects the routes every module declared for one mount point.
 *
 * Modules are visited in registry order. Sibling routes that claim the same path
 * segment fold into one route, so two modules can own different pages under one
 * segment without importing each other.
 */
export const routesFor = (modules: readonly ScyllaModule[], mount: RouteMount): AppRoute[] =>
  mergeSharedParents(
    modules.flatMap(module =>
      (module.routes ?? []).filter(route => route.mount === mount).map(toAppRoute),
    ),
  );
