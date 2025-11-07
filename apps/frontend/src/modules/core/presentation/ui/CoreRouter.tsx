// CoreRouter.tsx
import { createBrowserRouter, Navigate } from 'react-router-dom';

import LoginPage from '@/modules/login/presentation/ui/LoginPage';
import UserSettingsPage from '@/modules/user_settings/presentation/ui/UserSettingsPage';
import MarketplacePage from '@/modules/marketplace/presentation/ui/MarketplacePage';
import { Layout } from '@/modules/core/presentation/ui/Layout';
import { MarketplaceTopBar } from '@/modules/marketplace/presentation/ui/MarketplaceTopBar.tsx';

export const coreRouter = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    element: <Layout />,
    children: [
      {
        path: '/user_settings',
        element: <UserSettingsPage />,
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
  {
    path: '*',
    element: <Navigate to='/login' replace />,
  },
]);
