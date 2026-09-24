import type { Component, Snippet } from 'svelte';
import type { BreadcrumbFn } from './crumb.struct.ts';
import type { RouteMount, RouteParams, RouteSource } from './scylla-module.struct.ts';

/** A component around every page of a mount, such as the app layout. It renders `children`. */
export type LayoutComponent = Component<{ children: Snippet }>;

/**
 * A component around every page of a mount, for example to sync the URL with
 * the context store. It gets the route parameters, and renders `children`.
 */
export type RouteWrapper = Component<{ params: RouteParams; children: Snippet }>;

/** What the shell attaches to a mount. */
export interface MountDefinition {
  /** The mount this one sits under. A mount without a parent starts at `/`. */
  parent?: RouteMount;
  /** Relative to the parent mount. */
  path?: string;
  /** Around every page of the mount. Only a root mount has one; its pages animate. */
  layout?: LayoutComponent;
  /** Around every page of the mount, inside the wrappers of its parents. */
  wrapper?: RouteWrapper;
  /** The crumb of the mount's own path. */
  breadcrumb?: BreadcrumbFn;
}

/** Everything the router is built from. */
export interface AppRouterConfig {
  mounts: Readonly<Record<RouteMount, MountDefinition>>;
  /** The route declarations, in registration order. */
  modules: readonly RouteSource[];
  /** Shows, without any layout, for a URL that no route matches. */
  fallback: Component;
}
