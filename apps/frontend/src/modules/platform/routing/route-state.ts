import type { RouteMeta } from 'sv-router';
import type { Permission } from '@platform/authz';
import type { RouteWrapper } from './app-route.struct.ts';
import type { RouteHandle } from './route-handle.struct.ts';
import type { PageLoader, RouteParams } from './scylla-module.struct.ts';

/** A route handle on the current URL, with the pathname of its route. */
export interface TrailCrumb {
  handle: RouteHandle;
  pathname: string;
}

/** The reactive state of the current route, as the router exposes it. */
export interface RouteState {
  readonly params: RouteParams;
  readonly pathname: string;
  readonly search: unknown;
  readonly meta: RouteMeta;
}

let route: RouteState | null = null;

/** Connects the accessors below to the router. `createAppRouter` calls it. */
export const setRouteState = (state: RouteState | null): void => {
  route = state;
};

/** The parameters of the current route. Reactive. */
export const routeParams = (): RouteParams => route?.params ?? {};

/** The current pathname. Reactive. */
export const routePathname = (): string => route?.pathname ?? window.location.pathname;

/** The page loader of the current route, if the route has a page. Reactive. */
export const routePage = (): PageLoader | undefined => route?.meta.page;

/** The redirect target of the current route, if the route is a redirect. Reactive. */
export const routeRedirect = (): string | undefined => route?.meta.redirect;

/** The wrappers around the page of the current route, from the outermost. Reactive. */
export const routeWrappers = (): readonly RouteWrapper[] => route?.meta.wrappers ?? [];

/** True when the current route shows inside the shell layout. Reactive. */
export const routeInShell = (): boolean => route?.meta.shell ?? false;

/** The handles on the current URL, from the outermost route to the page. Reactive. */
export const routeTrail = (): TrailCrumb[] => {
  const marks = route?.meta.trail ?? [];
  const segments = routePathname().split('/').filter(Boolean);

  return marks.map(({ handle, depth }) => ({
    handle,
    pathname: `/${segments.slice(0, depth).join('/')}`,
  }));
};

/** The permission of the deepest route on the current URL that declares one. Reactive. */
export const requiredPermission = (): Permission | undefined =>
  routeTrail()
    .map(crumb => crumb.handle.permission)
    .filter(permission => permission !== undefined)
    .at(-1);
