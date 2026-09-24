import { setAppNavigator } from '@platform/context';
import { createAppRouter, type AppRouterConfig, type RouteSource } from '@platform/routing';
import { modules } from '@core/di/registry.ts';
import AppShell from './AppShell.svelte';
import ContextCleanerWrapper from './ContextCleaner.wrapper.svelte';
import LoginRedirect from './LoginRedirect.svelte';
import OrganizationSyncWrapper from './OrganizationSync.wrapper.svelte';
import { coreMessages } from './core.messages.ts';

/** The pages the shell owns: the entry points that send the user on to a module's page. */
const shellRoutes: RouteSource = {
  id: 'shell',
  routes: {
    app: [{ page: () => import('./OrganizationRedirect.wrapper.svelte') }],
    organization: [{ redirect: 'dashboard' }],
  },
};

/**
 * The application: the mounts the shell owns, and the routes that the modules
 * graft onto them.
 *
 * Add a page in the `*.module.ts` of its module, not here. This changes only when
 * the shell gets a new mount.
 */
export const appRoutes: AppRouterConfig = {
  mounts: {
    public: {},
    app: { layout: AppShell },
    organization: { parent: 'app', path: ':organizationSlug', wrapper: OrganizationSyncWrapper },
    project: {
      parent: 'organization',
      path: 'projects/:projectId',
      wrapper: ContextCleanerWrapper,
      breadcrumb: ({ projectName }) => ({ label: coreMessages.project, highlight: projectName }),
    },
  },
  modules: [shellRoutes, ...modules],
  fallback: LoginRedirect,
};

/** Starts the router and installs it as the navigator of the app. Call it once, before mount. */
export const startRouter = (): void => {
  setAppNavigator(createAppRouter(appRoutes));
};
