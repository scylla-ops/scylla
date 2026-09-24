import type { Component } from 'svelte';
import type { Permission } from '@platform/authz';
import type {
  AppRouterConfig,
  LayoutComponent,
  MountDefinition,
  RouteWrapper,
} from '../declaration/app-router-config.struct.ts';
import type { BreadcrumbFn } from '../declaration/crumb.struct.ts';
import type { PageLoader, RouteMount } from '../declaration/scylla-module.struct.ts';
import { flattenModuleRoutes, mergeSamePath, type FlatRoute } from './flatten-routes.ts';
import {
  bySpecificity,
  joinPath,
  pathKey,
  splitPath,
  startsWithPath,
  type RoutePath,
} from './route-path.ts';

/** A crumb on the way to a page, and the number of URL segments its path covers. */
export interface TrailMark {
  breadcrumb: BreadcrumbFn;
  depth: number;
}

/** A route the router can match: a page or a redirect, with all it needs to render. */
export interface CompiledRoute {
  mount: RouteMount;
  /** From the root, e.g. `[':organizationSlug', 'agents']`. */
  path: RoutePath;
  page?: PageLoader;
  redirect?: string;
  permission?: Permission;
  /** The layout of the root mount. A route without one renders alone. */
  layout?: LayoutComponent;
  /** The wrappers of the mount and of its parents, from the outermost. */
  wrappers: readonly RouteWrapper[];
  /** The crumbs of every path that leads here, from the root. */
  trail: readonly TrailMark[];
}

/** The compiled router: its routes, most specific first, and the page for no match. */
export interface RouteTable {
  routes: readonly CompiledRoute[];
  fallback: Component;
}

type Mounts = AppRouterConfig['mounts'];

/** The mount and its parents, from the root. */
export const mountChain = (mount: RouteMount, mounts: Mounts): MountDefinition[] => {
  const { parent } = mounts[mount];
  return [...(parent ? mountChain(parent, mounts) : []), mounts[mount]];
};

/** The path of a mount from the root, e.g. `project` → `/:organizationSlug/projects/:projectId`. */
export const mountPath = (mount: RouteMount, mounts: Mounts): RoutePath =>
  mountChain(mount, mounts).flatMap(definition => splitPath(definition.path));

/** The crumb a mount declares on its own path, as a route of that mount. */
const mountCrumbs = (mounts: Mounts): FlatRoute[] =>
  (Object.keys(mounts) as RouteMount[]).flatMap(mount => {
    const { breadcrumb } = mounts[mount];
    return breadcrumb ? [{ mount, path: [], breadcrumb, declaredBy: [`mount "${mount}"`] }] : [];
  });

/** Moves a route from its mount to the root: `agents` → `:organizationSlug/agents`. */
const placeInMount = (route: FlatRoute, mounts: Mounts): FlatRoute => ({
  ...route,
  path: [...mountPath(route.mount, mounts), ...route.path],
});

/**
 * The crumbs of a page: those of every path its own path starts with, from the
 * root. So the page of `jobs/:jobId` shows the crumb that another module
 * declares on `jobs`.
 */
export const trailOf = (
  path: RoutePath,
  crumbs: readonly { path: RoutePath; breadcrumb: BreadcrumbFn }[],
): TrailMark[] =>
  crumbs
    .filter(crumb => startsWithPath(path, crumb.path))
    .sort((a, b) => a.path.length - b.path.length)
    .map(crumb => ({ breadcrumb: crumb.breadcrumb, depth: crumb.path.length }));

const assertUnique = (routes: readonly CompiledRoute[]): void => {
  const seen = new Set<string>();
  for (const { path } of routes) {
    const key = pathKey(path);
    if (seen.has(key)) throw new Error(`Two mounts declare a page on "${joinPath(path)}".`);
    seen.add(key);
  }
};

/**
 * Turns the route declarations into the flat table the router matches against.
 *
 * 1. `flattenModuleRoutes` unfolds each module's tree and merges what several
 *    modules — and the mounts, for their crumbs — declare on one path.
 * 2. `placeInMount` gives each route its full path from the root.
 * 3. Each page or redirect becomes a `CompiledRoute`, with the layout and the
 *    wrappers of its mount and the crumbs of its `trailOf`.
 * 4. The routes are sorted `bySpecificity`, so the first match is the right one.
 *
 * It throws on a declaration that cannot be routed, so a mistake fails at startup
 * and in the tests rather than on the page.
 */
export const compileRoutes = ({ mounts, modules, fallback }: AppRouterConfig): RouteTable => {
  const declared = mergeSamePath([...mountCrumbs(mounts), ...flattenModuleRoutes(modules)]).map(
    route => placeInMount(route, mounts),
  );
  const crumbs = declared.flatMap(({ path, breadcrumb }) =>
    breadcrumb ? [{ path, breadcrumb }] : [],
  );

  const routes: CompiledRoute[] = declared
    .filter(route => route.page || route.redirect)
    .map(({ mount, path, page, redirect, permission }) => {
      const chain = mountChain(mount, mounts);
      return {
        mount,
        path,
        page,
        redirect,
        permission,
        layout: chain[0].layout,
        wrappers: chain.flatMap(definition => (definition.wrapper ? [definition.wrapper] : [])),
        trail: trailOf(path, crumbs),
      };
    })
    .sort((a, b) => bySpecificity(a.path, b.path));

  assertUnique(routes);
  return { routes, fallback };
};
