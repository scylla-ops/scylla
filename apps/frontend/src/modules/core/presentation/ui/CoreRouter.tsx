// CoreRouter.tsx
import { createBrowserRouter, Navigate } from 'react-router-dom';
import LoginPage from '@/modules/login/presentation/ui/LoginPage';
import UserSettingsPage from '@/modules/user_settings/presentation/ui/UserSettingsPage';
import MarketplacePage from '@/modules/marketplace/presentation/ui/MarketplacePage';
import { Layout } from '@/modules/core/presentation/ui/Layout';
import { MarketplaceTopBar } from '@/modules/marketplace/presentation/ui/MarketplaceTopBar';
import { RequireAuth } from '@core/presentation/ui/RequireAuth.tsx';
import { PipelineCreationPage } from '@/modules/pipeline-creation/presentation/ui/PipelineCreationPage.tsx';
import { PipelineCreationTopbar } from '@/modules/pipeline-creation/presentation/ui/PipelineCreationTopbar.tsx';
import { DashboardPipelinePage } from '@/modules/pipeline-dashboard/presentation/ui/DashboardPipelinePage';

export const coreRouter = createBrowserRouter([
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
            path: '/pipeline-creation',
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
    element: <Navigate to='/login' replace />,
  },
]);
