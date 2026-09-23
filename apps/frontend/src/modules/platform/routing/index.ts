/**
 * How a module declares itself to the application that assembles it, and how
 * the shell turns those declarations into a router.
 *
 * Sits in `platform/` because both sides need it: features declare routes,
 * `core` composes them and `layout` reads the breadcrumb contract — none of
 * which may depend on each other. This is also the only module that knows the
 * router library.
 */
export type {
  ScyllaModule,
  ModuleRoute,
  NavEntry,
  PageComponent,
  PageLoader,
  RouteMount,
  RouteParams,
} from './scylla-module.struct.ts';
export type { RouteHandle, Crumb, BreadcrumbParams } from './route-handle.struct.ts';
export type {
  AppRoute,
  AppRouterConfig,
  LayoutComponent,
  RouteWrapper,
} from './app-route.struct.ts';
export { routesFor, navEntriesFor } from './compose-module-routes.ts';
export { createAppRouter } from './app-router.ts';
export {
  requiredPermission,
  routeParams,
  routePathname,
  routeTrail,
  type TrailCrumb,
} from './route-state.ts';
export { default as Redirect } from './Redirect.svelte';
export { Router as RouterView } from 'sv-router';
