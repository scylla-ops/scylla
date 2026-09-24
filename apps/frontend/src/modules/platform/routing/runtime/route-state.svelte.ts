import type { BreadcrumbFn } from '../declaration/crumb.struct.ts';
import type { RouteParams } from '../declaration/scylla-module.struct.ts';
import type { RouteTable } from '../compilation/compile-routes.ts';
import { joinPath, splitPath } from '../compilation/route-path.ts';
import { location } from './location.svelte.ts';
import { matchRoute, type RouteMatch } from './match-route.ts';

/** A crumb on the current URL, with the pathname it links to. */
export interface TrailCrumb {
  breadcrumb: BreadcrumbFn;
  pathname: string;
}

let table = $state.raw<RouteTable | null>(null);

/** Installs the routes that the accessors below match against. `createAppRouter` calls it. */
export const setRouteTable = (next: RouteTable | null): void => {
  table = next;
};

/** The page to show when no route matches. */
export const routeFallback = () => table?.fallback;

/** The route of the current URL and its parameters, or `null`. Reactive. */
export const currentMatch = (): RouteMatch | null =>
  table && matchRoute(table.routes, location.pathname);

/** The parameters of the current route. Reactive. */
export const routeParams = (): RouteParams => currentMatch()?.params ?? {};

/** The current pathname. Reactive once the router is created. */
export const routePathname = (): string => (table ? location.pathname : window.location.pathname);

/** The crumbs of the current URL, from the root to the page. Reactive. */
export const routeTrail = (): TrailCrumb[] => {
  const segments = splitPath(location.pathname);

  return (currentMatch()?.route.trail ?? []).map(({ breadcrumb, depth }) => ({
    breadcrumb,
    pathname: joinPath(segments.slice(0, depth)),
  }));
};
