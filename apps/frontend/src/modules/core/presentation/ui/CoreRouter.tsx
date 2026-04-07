// CoreRouter.tsx
import { createBrowserRouter, Navigate } from 'react-router-dom';
import LoginPage from '@/modules/features/login/presentation/ui/LoginPage';
import UserSettingsPage from '@/modules/features/user_settings/presentation/ui/UserSettingsPage';
import MarketplacePage from '@/modules/features/marketplace/presentation/ui/MarketplacePage';
import { Layout } from '@/modules/layout/presentation/ui/Layout.tsx';
import { MarketplaceTopBar } from '@/modules/features/marketplace/presentation/ui/MarketplaceTopBar';
import { RequireAuth } from '@core/presentation/ui/RequireAuth.tsx';
import { PipelineCreationPage } from '@/modules/features/pipeline-creation/presentation/ui/PipelineCreationPage.tsx';
import { PipelineCreationTopbar } from '@/modules/features/pipeline-creation/presentation/ui/PipelineCreationTopbar.tsx';
import { DashboardPipelinePage } from '@/modules/features/pipeline-dashboard/presentation/ui/DashboardPipelinePage';
import { PipelineDashboardTopBar } from '@/modules/features/pipeline-dashboard/presentation/ui/PipelineDashboardTopBar.tsx';
import ProjectPage from '@/modules/features/project/presentation/ui/ProjectPage.tsx';
import { ProjectTopBar } from '@/modules/features/project/presentation/ui/ProjectTopBar.tsx';

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
            path: '/projects/:projectId/',
            children: [
              {
                index: true,
                element: <DashboardPipelinePage />,
                handle: { topbar: <PipelineDashboardTopBar /> },
              },
              {
                path: 'create',
                element: <PipelineCreationPage />,
                handle: {
                  topbar: <PipelineCreationTopbar />,
                  tabsDefaultValue: 'scripting',
                },
              },
              {
                path: 'edit/:pipelineId',
                element: <PipelineCreationPage />,
                handle: {
                  topbar: <PipelineCreationTopbar />,
                  tabsDefaultValue: 'scripting',
                },
              },
            ],
          },
          {
            path: '/marketplace',
            element: <MarketplacePage />,
            handle: {
              topbar: <MarketplaceTopBar />,
            },
          },
          {
            path: '/projects',
            element: <ProjectPage />,
            handle: {
              topbar: <ProjectTopBar />,
            },
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
