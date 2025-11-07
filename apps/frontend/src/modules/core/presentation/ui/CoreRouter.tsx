// CoreRouter.tsx
import { createBrowserRouter, Navigate } from 'react-router-dom';

import LoginPage from '@/modules/login/presentation/ui/LoginPage';
import UserSettingsPage from '@/modules/user_settings/presentation/ui/UserSettingsPage';
import MarketplacePage from '@/modules/marketplace/presentation/ui/MarketplacePage';
import { Layout } from '@/modules/core/presentation/ui/Layout';

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
        handle: {
          topbar: <div> test </div>,
        },
      },
      {
        path: '/marketplace',
        element: <MarketplacePage />,
        handle: {
          topbar: <div />,
        },
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to='/login' replace />,
  },
]);
