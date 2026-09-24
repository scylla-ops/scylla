import type { AppNavigator, NavigateOptions } from '@platform/context';
import type { AppRouterConfig } from '../declaration/app-router-config.struct.ts';
import { compileRoutes } from '../compilation/compile-routes.ts';
import { changeLocation, location, syncLocation } from './location.svelte.ts';
import { resolveTarget } from './resolve-target.ts';
import { setRouteTable } from './route-state.svelte.ts';

/**
 * Compiles the routes and starts the router on them.
 *
 * Returns the navigator that the shell installs with `setAppNavigator`. Call it
 * once, before the app mounts `RouterView`. It throws when a route declaration
 * is wrong, see `compileRoutes`.
 */
export const createAppRouter = (config: AppRouterConfig): AppNavigator => {
  setRouteTable(compileRoutes(config));
  syncLocation();

  return {
    navigate: (to: string, options?: NavigateOptions) => {
      const { pathname, search, hash } = resolveTarget(to, location.pathname);
      changeLocation(pathname + search + hash, options);
    },
    back: () => history.back(),
    pathname: () => location.pathname,
    search: () => location.search,
  };
};
