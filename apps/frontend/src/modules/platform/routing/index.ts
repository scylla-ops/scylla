/**
 * How a module declares its routes, and the router the shell builds from them.
 *
 * Sits in `platform/` because both sides need it: features declare routes,
 * `core` builds the router and `layout` reads the sidebar and breadcrumb
 * contracts — none of which may depend on each other.
 */
export type {
  ScyllaModule,
  ModuleRoute,
  ModuleRoutes,
  NavLink,
  PageComponent,
  PageLoader,
  RouteMount,
  RouteParams,
  RouteSource,
} from './declaration/scylla-module.struct.ts';
export type { BreadcrumbFn, BreadcrumbParams, Crumb } from './declaration/crumb.struct.ts';
export type {
  AppRouterConfig,
  LayoutComponent,
  MountDefinition,
  RouteWrapper,
} from './declaration/app-router-config.struct.ts';
export {
  compileRoutes,
  type CompiledRoute,
  type RouteTable,
} from './compilation/compile-routes.ts';
export { navEntriesFor, type NavEntry } from './compilation/nav-entries.ts';
export { createAppRouter } from './runtime/app-router.ts';
export {
  routeParams,
  routePathname,
  routeTrail,
  type TrailCrumb,
} from './runtime/route-state.svelte.ts';
export { default as Redirect } from './view/Redirect.svelte';
export { default as RouterView } from './view/RouterView.svelte';
