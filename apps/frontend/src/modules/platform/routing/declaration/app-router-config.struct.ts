import type { Component, Snippet } from 'svelte';
import type { BreadcrumbFn } from './crumb.struct.ts';
import type { RouteMount, RouteParams, RouteSource } from './scylla-module.struct.ts';

export type LayoutComponent = Component<{ children: Snippet }>;

/** Around every page of a mount, e.g. to sync the URL with the context store. */
export type RouteWrapper = Component<{ params: RouteParams; children: Snippet }>;

export interface MountDefinition {
  /** Without a parent, the mount starts at `/`. */
  parent?: RouteMount;
  path?: string;
  /** Only on a root mount. Its pages animate. */
  layout?: LayoutComponent;
  /** Inside the wrappers of the parent mounts. */
  wrapper?: RouteWrapper;
  breadcrumb?: BreadcrumbFn;
}

export interface AppRouterConfig {
  mounts: Readonly<Record<RouteMount, MountDefinition>>;
  modules: readonly RouteSource[];
  /** Rendered without any layout. */
  fallback: Component;
}
