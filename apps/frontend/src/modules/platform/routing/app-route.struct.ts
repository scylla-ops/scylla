import type { Component, Snippet } from 'svelte';
import type { RouteHandle } from './route-handle.struct.ts';
import type { PageComponent, PageLoader, RouteParams } from './scylla-module.struct.ts';

/** The layout around every route of the shell. It renders `children` where the page goes. */
export type LayoutComponent = Component<{ children: Snippet }>;

/**
 * A component around every page below a route, for example to sync the URL with
 * the context store. It gets the route parameters, and renders `children`.
 */
export type RouteWrapper = Component<{ params: RouteParams; children: Snippet }>;

/**
 * One node of the route tree that the router mounts.
 *
 * `routesFor` makes these from the module declarations. The shell adds its own
 * nodes around them: a `wrapper` goes around every page below the node, a
 * `component` is an eager shell page, and a `redirect` sends the user to another
 * URL. A relative `redirect` starts from the URL of the route.
 */
export interface AppRoute {
  path?: string;
  index?: boolean;
  handle?: RouteHandle;
  lazy?: PageLoader;
  component?: PageComponent;
  redirect?: string;
  wrapper?: RouteWrapper;
  children?: AppRoute[];
}

/** The complete route tree of the application. */
export interface AppRouterConfig {
  /** Routes that show without the shell, for example `/login`. */
  publicRoutes: readonly AppRoute[];
  /** The layout around all other routes. */
  shell: LayoutComponent;
  routes: readonly AppRoute[];
  /** Shows for a URL that no route matches, without the shell. */
  fallback: Component;
}
