import { createBrowserRouter, Navigate } from 'react-router-dom';
import { msg } from '@lingui/core/macro';
import { Layout } from '@/modules/layout/presentation/ui/Layout.tsx';
import { AuthGuard } from '@core/presentation/ui/router/Auth.guard.tsx';
import { RouteGuard } from '@platform/routing/RouteGuard.tsx';
import { navEntriesFor, routesFor } from '@platform/routing/compose-module-routes.ts';
import { modules } from '@core/di/registry.ts';
import { ContextCleanerWrapper } from './ContextCleaner.wrapper.tsx';
import { OrganizationSyncWrapper } from './OrganizationSync.wrapper.tsx';
import { OrganizationRedirectWrapper } from './OrganizationRedirect.wrapper.tsx';

/**
 * The application shell.
 *
 * It owns only the skeleton — authentication, layout, and the wrappers that keep
 * the active organization and project in sync — and grafts each module's own
 * route declarations at the scope they asked for. Adding a page is a change to
 * one module's `di/*.module.ts`; this file does not move.
 *
 * `RouteGuard` is a pathless layout route per scope: it reads the `permission`
 * a route declared and applies `RequirePermission` once, instead of every page
 * wrapping its own element.
 */
export const CoreRouter = createBrowserRouter([
  ...routesFor(modules, 'public'),

  {
    element: <AuthGuard />,
    children: [
      {
        element: <Layout navEntries={navEntriesFor(modules)} />,
        children: [
          {
            index: true,
            element: <OrganizationRedirectWrapper />,
          },
          {
            path: '/:organizationSlug',
            element: <OrganizationSyncWrapper />,
            children: [
              {
                element: <RouteGuard />,
                children: [
                  ...routesFor(modules, 'organization'),
                  {
                    path: 'projects',
                    handle: { breadcrumb: () => ({ label: msg`Projects` }) },
                    children: [
                      ...routesFor(modules, 'projects'),
                      {
                        //TODO: add a loader here to resolve project and pipeline
                        // names from ids for the breadcrumbs
                        path: ':projectId',
                        element: <ContextCleanerWrapper />,
                        handle: {
                          breadcrumb: ({ projectName }: { projectName?: string }) => ({
                            label: msg`Project`,
                            highlight: projectName,
                          }),
                        },
                        children: [
                          {
                            element: <RouteGuard />,
                            children: routesFor(modules, 'project'),
                          },
                        ],
                      },
                    ],
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  },

  {
    path: '*',
    element: <Navigate to='/login' replace />,
  },
]);
