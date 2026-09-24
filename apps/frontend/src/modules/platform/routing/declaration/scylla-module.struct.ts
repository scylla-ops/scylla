import type { Component } from 'svelte';
import type { MessageDescriptor } from '@lingui/core';
import type { Permission } from '@platform/authz';
import type { LucideIcon } from '@shared/presentation/ui/icon.ts';
import type { BreadcrumbFn } from './crumb.struct.ts';

/** Always strings. A page declares the ones it reads as optional props. */
export type RouteParams = Record<string, string | undefined>;

export type PageComponent = Component<RouteParams> | Component<Record<string, never>>;

/** Write it `() => import('./X.page.svelte')`: the page keeps its own chunk. */
export type PageLoader = () => Promise<{ default: PageComponent }>;

/** Where a module's routes graft. The shell gives each mount its path, layout and wrappers. */
export type RouteMount =
  | 'public'
  /** Inside the shell, above any organization. */
  | 'app'
  | 'organization'
  | 'project';

/** Its URL and its permission are the route's. */
export interface NavLink {
  section: 'organization' | 'system';
  title: MessageDescriptor;
  icon?: LucideIcon;
  order?: number;
}

/** A route with only `path` and `children` groups them; a child without `path` is its parent's page. */
export interface ModuleRoute {
  /** Relative to the parent route or to the mount. May hold several segments. */
  path?: string;
  /** Keep it lazy. */
  page?: PageLoader;
  /** A relative target starts from this route's URL. */
  redirect?: string;
  /** Read by the route guard and the sidebar. Guards this page only: children declare their own. */
  permission?: Permission;
  /** Shows on this page and on every page below its path. */
  breadcrumb?: BreadcrumbFn;
  /** Only on `organization` routes. */
  nav?: NavLink;
  children?: readonly ModuleRoute[];
}

export type ModuleRoutes = Partial<Record<RouteMount, readonly ModuleRoute[]>>;

/** Declared in `<feature>.module.ts`, never exported from the module's `index.ts`. */
export interface ScyllaModule<TDomain extends object = object> {
  readonly id: string;
  readonly domain: TDomain;
  readonly routes?: ModuleRoutes;
}

/** What the router reads of a module. The shell declares its own routes with it. */
export type RouteSource = Pick<ScyllaModule, 'id' | 'routes'>;
