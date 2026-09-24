import type { AppNavigator, NavigateOptions } from '@platform/context';
import type { AppRouterConfig } from '../declaration/app-router-config.struct.ts';
import { compileRoutes } from '../compilation/compile-routes.ts';
import { changeLocation, location, syncLocation } from './location.svelte.ts';
import { resolveTarget } from './resolve-target.ts';
import { setRouteTable } from './route-state.svelte.ts';

/** Call it once, before `RouterView` mounts. Throws when a route declaration is wrong. */
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
