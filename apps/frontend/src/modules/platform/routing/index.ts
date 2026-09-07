/**
 * How a module declares itself to the application that assembles it, and how
 * the shell turns those declarations into a router.
 *
 * Sits in `platform/` because both sides need it: features declare routes,
 * `core` composes them and `layout` reads the breadcrumb contract — none of
 * which may depend on each other.
 */
export type {
  ScyllaModule,
  ModuleRoute,
  NavEntry,
  RouteMount,
} from './scylla-module.struct.ts';
export type { RouteHandle, Crumb, BreadcrumbParams } from './route-handle.struct.ts';
export { routesFor, navEntriesFor } from './compose-module-routes.ts';
export { RouteGuard } from './RouteGuard.tsx';
