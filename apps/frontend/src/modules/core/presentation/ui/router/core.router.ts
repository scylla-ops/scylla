import { setAppNavigator } from '@platform/context';
import {
  createAppRouter,
  routesFor,
  type AppRouterConfig,
  type BreadcrumbParams,
} from '@platform/routing';
import { modules } from '@core/di/registry.ts';
import AppShell from './AppShell.svelte';
import ContextCleanerWrapper from './ContextCleaner.wrapper.svelte';
import LoginRedirect from './LoginRedirect.svelte';
import OrganizationRedirectWrapper from './OrganizationRedirect.wrapper.svelte';
import OrganizationSyncWrapper from './OrganizationSync.wrapper.svelte';
import { coreMessages } from './core.messages.ts';

/**
 * The application shell: the skeleton of the routes, with the routes of each
 * module grafted at the mount that it asked for.
 *
 * Add a page in the `*.module.ts` of its module, not here. This tree changes only
 * when the shell gets a new mount point.
 */
export const appRoutes: AppRouterConfig = {
  publicRoutes: routesFor(modules, 'public'),
  shell: AppShell,
  fallback: LoginRedirect,
  routes: [
    { index: true, component: OrganizationRedirectWrapper },
    {
      path: ':organizationSlug',
      wrapper: OrganizationSyncWrapper,
      children: [
        { index: true, redirect: 'dashboard' },
        ...routesFor(modules, 'organization'),
        {
          path: 'projects',
          handle: { breadcrumb: () => ({ label: coreMessages.projects }) },
          children: [
            ...routesFor(modules, 'projects'),
            {
              path: ':projectId',
              wrapper: ContextCleanerWrapper,
              handle: {
                breadcrumb: ({ projectName }: BreadcrumbParams) => ({
                  label: coreMessages.project,
                  highlight: projectName,
                }),
              },
              children: routesFor(modules, 'project'),
            },
          ],
        },
      ],
    },
  ],
};

/** Starts the router and installs it as the navigator of the app. Call it once, before mount. */
export const startRouter = (): void => {
  setAppNavigator(createAppRouter(appRoutes));
};
