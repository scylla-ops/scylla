import type { RouteObject } from 'react-router-dom';
import type { LucideIcon } from 'lucide-react';
import type { MessageDescriptor } from '@lingui/core';
import type { Permission } from '@platform/authz/domain/structs/permission.struct.ts';
import type { BreadcrumbParams, Crumb } from './route-handle.struct.ts';

/**
 * Where in the app shell a module's routes are grafted.
 *
 * The shell owns the skeleton — auth guard, layout, and the wrappers that sync
 * the active organization and project — and modules say which scope they belong
 * to rather than restating that nesting themselves.
 */
export type RouteMount =
  /** Outside the auth guard, e.g. `/login`. */
  | 'public'
  /** Under `/:organizationSlug`. */
  | 'organization'
  /** Under `/:organizationSlug/projects`. */
  | 'projects'
  /** Under `/:organizationSlug/projects/:projectId`. */
  | 'project';

export interface ModuleRoute {
  mount: RouteMount;
  path?: string;
  index?: boolean;
  /**
   * Required to enter the route. Declared once and read by both the shell's
   * route guard and the sidebar, so a link can no longer be visible for a page
   * that will deny you — or hidden for one that would not.
   */
  permission?: Permission;
  breadcrumb?: (params: BreadcrumbParams) => Crumb;
  /**
   * react-router's lazy loader. Keeping page components behind it is what lets
   * the registry stay eager (use-case classes only) while the UI is split per
   * route.
   *
   * Omitted for a pure grouping route — one that exists only to own a path
   * segment, its breadcrumb and its children (react-router renders an `Outlet`).
   */
  lazy?: NonNullable<RouteObject['lazy']>;
  children?: ModuleRoute[];
}

export interface NavEntry {
  /** Which sidebar card the entry belongs to. */
  section: 'organization' | 'system';
  /** Lazily evaluated so the label follows a locale switch. */
  title: MessageDescriptor;
  /** Appended to the current organization prefix. */
  url: string;
  icon?: LucideIcon;
  permission?: Permission;
  /** Lower sorts first within a section. */
  order?: number;
}

/**
 * What a module exposes to the application that assembles it.
 *
 * `domain` is the dependency-injection surface; `routes` and `nav` let the
 * router and the sidebar be derived from one declaration instead of the three
 * hand-maintained lists that used to drift apart.
 *
 * Declared in `di/<feature>.module.ts` — deliberately *not* in the module's
 * `index.ts` public API, because the registry imports every module eagerly and a
 * barrel that also re-exports UI would pull every page back into the initial
 * chunk.
 */
export interface ScyllaModule<TDomain extends object = object> {
  readonly id: string;
  readonly domain: TDomain;
  readonly routes?: readonly ModuleRoute[];
  readonly nav?: readonly NavEntry[];
}
