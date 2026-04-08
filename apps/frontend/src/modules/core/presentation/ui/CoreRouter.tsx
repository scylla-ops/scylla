// CoreRouter.tsx
import { createBrowserRouter, Navigate } from 'react-router-dom';
import LoginPage from '@/modules/features/login/presentation/ui/LoginPage';
import UserSettingsPage from '@/modules/features/user_settings/presentation/ui/UserSettingsPage';
import MarketplacePage from '@/modules/features/marketplace/presentation/ui/MarketplacePage';
import { Layout } from '@/modules/layout/presentation/ui/Layout.tsx';
import { RequireAuth } from '@/modules/core/presentation/ui/middlewares/RequireAuth';
import { PipelineCreationPage } from '@/modules/features/pipeline-creation/presentation/ui/PipelineCreationPage.tsx';
import { DashboardPipelinePage } from '@/modules/features/pipeline-dashboard/presentation/ui/DashboardPipelinePage';
import ProjectPage from '@/modules/features/project/presentation/ui/ProjectPage.tsx';
import type { BreadcrumbParams } from '@core/presentation/models/RouteHandle.ts';
import { ContextCleaner } from './middlewares/ContextCleaner';

//TODO: put each navigations part in a separate file, (module ?)
export const CoreRouter = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },

  {
    element: <RequireAuth />,
    children: [
      {
        element: <Layout />,
        children: [
          {
            path: '/user-settings',
            element: <UserSettingsPage />,
          },
          {
            path: '/projects',
            handle: {
              breadcrumb: () => 'Projects',
            },
            children: [
              {
                index: true,
                element: <ProjectPage />,
              },
              {
                element: <ContextCleaner />,
                //TODO: here add loader: <ContextLoader/> used to fetch the project and pipeline names from ids for the breadcrumbs
                path: ':projectId',
                handle: {
                  breadcrumb: (params: BreadcrumbParams) => `Project #${params.projectName}`,
                },
                children: [
                  {
                    index: true,
                    element: <DashboardPipelinePage />,
                  },
                  {
                    path: 'create',
                    element: <PipelineCreationPage />,
                    handle: {
                      breadcrumb: () => `Create`,
                    },
                  },
                  {
                    path: 'edit/:pipelineId',
                    element: <PipelineCreationPage />,
                    handle: {
                      breadcrumb: ({ pipelineName }: BreadcrumbParams) =>
                        `Pipeline #${pipelineName}`,
                    },
                  },
                ],
              },
            ],
          },
          {
            path: '/marketplace',
            element: <MarketplacePage />,
          },
        ],
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to='/user-settings' replace />,
  },
]);
