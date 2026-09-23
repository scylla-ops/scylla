import type { Component } from 'svelte';
import type { AppRoute, AppRouterConfig, RouteWrapper } from './app-route.struct.ts';
import type { RouteHandle } from './route-handle.struct.ts';
import type { PageComponent, PageLoader } from './scylla-module.struct.ts';

/** A route handle, and the number of URL segments that its route covers. */
export interface RouteMark {
  handle: RouteHandle;
  depth: number;
}

declare module 'sv-router' {
  interface RouteMeta {
    page?: PageLoader;
    trail?: readonly RouteMark[];
    wrappers?: readonly RouteWrapper[];
    redirect?: string;
    shell?: boolean;
  }
}

export type RouterTree = Record<string, unknown>;

interface Ancestry {
  trail: readonly RouteMark[];
  wrappers: readonly RouteWrapper[];
  depth: number;
  shell: boolean;
}

const segmentsOf = (path = ''): string[] => path.split('/').filter(Boolean);

const keyOf = (route: AppRoute): string =>
  route.index ? '/' : `/${segmentsOf(route.path).join('/')}`;

const outsideShell = (key: string): string => key.replace(/\/([^/]+)$/, '/($1)');

const eager =
  (component: PageComponent): PageLoader =>
  () =>
    Promise.resolve({ default: component });

const toNode = (route: AppRoute, parent: Ancestry, page: Component): unknown => {
  const depth = parent.depth + segmentsOf(route.path).length;
  const ancestry: Ancestry = {
    trail: route.handle ? [...parent.trail, { handle: route.handle, depth }] : parent.trail,
    wrappers: route.wrapper ? [...parent.wrappers, route.wrapper] : parent.wrappers,
    depth,
    shell: parent.shell,
  };

  if (route.children) return toLevel(route.children, ancestry, page);

  const load = route.lazy ?? (route.component && eager(route.component));

  return {
    '/': page,
    meta: {
      page: load,
      redirect: route.redirect,
      trail: ancestry.trail,
      wrappers: ancestry.wrappers,
      shell: ancestry.shell,
    },
  };
};

const toLevel = (routes: readonly AppRoute[], parent: Ancestry, page: Component): RouterTree =>
  Object.fromEntries(routes.map(route => [keyOf(route), toNode(route, parent, page)]));

const root = (shell: boolean): Ancestry => ({ trail: [], wrappers: [], depth: 0, shell });

/**
 * Converts the application routes into the route object of `sv-router`.
 *
 * Every page renders through `page`, which reads the page loader, the handles
 * and the wrappers from the route metadata. The public routes and the fallback
 * break out of the shell layout.
 */
export const toRouterTree = (
  { publicRoutes, shell, routes, fallback }: AppRouterConfig,
  page: Component,
): RouterTree => ({
  ...toLevel(routes, root(true), page),
  ...Object.fromEntries(
    publicRoutes.map(route => [outsideShell(keyOf(route)), toNode(route, root(false), page)]),
  ),
  '(*)': fallback,
  layout: shell,
});
