// CoreRouter.tsx
import { createBrowserRouter, Navigate } from 'react-router-dom';
import LoginPage from '@/modules/features/login/presentation/ui/LoginPage';
import UserSettingsPage from '@/modules/features/user_settings/presentation/ui/UserSettingsPage';
import MarketplacePage from '@/modules/features/marketplace/presentation/ui/MarketplacePage';
import { Layout } from '@/modules/layout/presentation/ui/Layout.tsx';
import { RequireAuth } from '@core/presentation/ui/RequireAuth.tsx';
import { PipelineCreationPage } from '@/modules/features/pipeline-creation/presentation/ui/PipelineCreationPage.tsx';
import { DashboardPipelinePage } from '@/modules/features/pipeline-dashboard/presentation/ui/DashboardPipelinePage';
import ProjectPage from '@/modules/features/project/presentation/ui/ProjectPage.tsx';
import type { BreadcrumbParams } from '@core/presentation/models/RouteHandle.ts';

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
                path: ':projectId',
                handle: {
                  breadcrumb: (params: BreadcrumbParams) => `Project #${params.projectId}`,
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
                      breadcrumb: ({ pipelineId }: BreadcrumbParams) => `Pipeline #${pipelineId}`,
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
