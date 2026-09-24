import type { Component } from 'svelte';
import type { MessageDescriptor } from '@lingui/core';
import type { Permission } from '@platform/authz';
import type { LucideIcon } from '@shared/presentation/ui/icon.ts';
import type { BreadcrumbFn } from './crumb.struct.ts';

/**
 * Route parameters, as the router gives them to a page: always strings.
 *
 * A page declares the parameters it reads as optional props.
 */
export type RouteParams = Record<string, string | undefined>;

/**
 * A page component. It takes no props, or it takes route parameters as optional
 * string props.
 */
export type PageComponent = Component<RouteParams> | Component<Record<string, never>>;

/**
 * Loads the page of a route. Write it as `() => import('./X.page.svelte')`, so that
 * the page stays in its own chunk.
 */
export type PageLoader = () => Promise<{ default: PageComponent }>;

/**
 * Where in the app a module's routes are grafted.
 *
 * The shell owns what a mount brings — its path, its layout, the wrappers that
 * sync the active organization and project. A module names the mount and never
 * restates that nesting.
 */
export type RouteMount =
  /** Outside the auth guard and the layout, e.g. `/login`. */
  | 'public'
  /** Inside the app shell, above any organization: `/`. */
  | 'app'
  /** Under `/:organizationSlug`. */
  | 'organization'
  /** Under `/:organizationSlug/projects/:projectId`. */
  | 'project';

/** The sidebar link of a route. Its URL and its permission are the route's own. */
export interface NavLink {
  /** Which sidebar card the entry belongs to. */
  section: 'organization' | 'system';
  /** A descriptor, so the label follows a locale switch. */
  title: MessageDescriptor;
  icon?: LucideIcon;
  /** Lower sorts first within a section. */
  order?: number;
}

/**
 * One route, as a module declares it.
 *
 * Every field is optional: a route with only `path` and `children` just groups
 * them under a segment, and a child without `path` is its parent's own page.
 */
export interface ModuleRoute {
  /** Relative to the parent route, or to the mount. May hold several segments. */
  path?: string;
  /** Loads the page. Keep it lazy: that is what keeps pages out of the entry chunk. */
  page?: PageLoader;
  /** Sends the user elsewhere. A relative target starts from this route's URL. */
  redirect?: string;
  /**
   * Required to open the page. Read by the route guard and by the sidebar link,
   * so the two cannot disagree. It guards this page only: children declare their own.
   */
  permission?: Permission;
  /** The crumb of this path. It shows on this page and on every page below it. */
  breadcrumb?: BreadcrumbFn;
  /** A sidebar link to this page. Only on `organization` routes. */
  nav?: NavLink;
  children?: readonly ModuleRoute[];
}

/** A module's routes, grouped by the mount they graft onto. */
export type ModuleRoutes = Partial<Record<RouteMount, readonly ModuleRoute[]>>;

/**
 * What a module exposes to the application that assembles it.
 *
 * `domain` is the dependency-injection surface; `routes` feeds the router, the
 * sidebar and the breadcrumbs from one declaration.
 *
 * Declared in `<feature>.module.ts` — deliberately *not* in the module's
 * `index.ts` public API, because the registry imports every module eagerly and a
 * barrel that also re-exports UI would pull every page back into the initial
 * chunk.
 */
export interface ScyllaModule<TDomain extends object = object> {
  readonly id: string;
  readonly domain: TDomain;
  readonly routes?: ModuleRoutes;
}

/** The part of a module the router reads. The shell declares its own routes with it. */
export type RouteSource = Pick<ScyllaModule, 'id' | 'routes'>;
