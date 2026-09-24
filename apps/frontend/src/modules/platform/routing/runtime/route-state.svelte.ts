import type { BreadcrumbFn } from '../declaration/crumb.struct.ts';
import type { RouteParams } from '../declaration/scylla-module.struct.ts';
import type { RouteTable } from '../compilation/compile-routes.ts';
import { joinPath, splitPath } from '../compilation/route-path.ts';
import { location } from './location.svelte.ts';
import { matchRoute, type RouteMatch } from './match-route.ts';

export interface TrailCrumb {
  breadcrumb: BreadcrumbFn;
  pathname: string;
}

let table = $state.raw<RouteTable | null>(null);

export const setRouteTable = (next: RouteTable | null): void => {
  table = next;
};

export const routeFallback = () => table?.fallback;

/** Reactive. */
export const currentMatch = (): RouteMatch | null =>
  table && matchRoute(table.routes, location.pathname);

export const routeParams = (): RouteParams => currentMatch()?.params ?? {};

/** Reactive once the router is created. */
export const routePathname = (): string => (table ? location.pathname : window.location.pathname);

export const routeTrail = (): TrailCrumb[] => {
  const segments = splitPath(location.pathname);

  return (currentMatch()?.route.trail ?? []).map(({ breadcrumb, depth }) => ({
    breadcrumb,
    pathname: joinPath(segments.slice(0, depth)),
  }));
};
