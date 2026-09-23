import { createRouter, type Routes } from 'sv-router';
import type { AppNavigator, NavigateOptions } from '@platform/context';
import type { AppRouterConfig } from './app-route.struct.ts';
import { resolveTarget } from './resolve-target.ts';
import { setRouteState, type RouteState } from './route-state.ts';
import { toRouterTree } from './router-tree.ts';
import RoutePage from './RoutePage.svelte';

type Navigate = (
  to: string | number,
  options?: { replace?: boolean; search?: string; hash?: string; scrollToTop?: false },
) => Promise<unknown>;

/**
 * Starts the router with the complete route tree.
 *
 * Returns the navigator that the shell installs with `setAppNavigator`. Call it
 * once, before the app mounts `RouterView`.
 */
export const createAppRouter = (config: AppRouterConfig): AppNavigator => {
  const router = createRouter(toRouterTree(config, RoutePage) as Routes);
  const navigate = router.navigate as unknown as Navigate;
  const state: RouteState = router.route;
  setRouteState(state);

  return {
    navigate: (to: string, options?: NavigateOptions) => {
      const target = resolveTarget(to, window.location.pathname);
      void navigate(target.pathname, {
        replace: options?.replace,
        search: target.search,
        hash: target.hash,
        scrollToTop: false,
      });
    },
    back: () => void navigate(-1),
    pathname: () => {
      void state.pathname;
      return window.location.pathname;
    },
    search: () => {
      void state.search;
      return window.location.search;
    },
  };
};
