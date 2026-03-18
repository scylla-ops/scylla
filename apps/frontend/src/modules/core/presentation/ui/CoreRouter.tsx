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
            path: '/pipeline-creation/:id?',
            element: <PipelineCreationPage />,
            handle: {
              topbar: <PipelineCreationTopbar />,
              tabsDefaultValue: 'scripting',
            },
          },
          {
            path: '/user-settings',
            element: <UserSettingsPage />,
          },
          {
            path: '/pipeline-dashboard',
            element: <DashboardPipelinePage />,
            handle: {
              topbar: <PipelineDashboardTopBar />,
            },
          },
          {
            path: '/marketplace',
            element: <MarketplacePage />,
            handle: {
              topbar: <MarketplaceTopBar />,
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
