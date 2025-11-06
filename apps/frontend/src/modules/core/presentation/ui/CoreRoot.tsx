import { BrowserRouter, Route, Navigate, Routes } from 'react-router-dom';
import LoginPage from '@/modules/login/presentation/ui/LoginPage.tsx';

import UserSettingsPage from '@/modules/user_settings/presentation/ui/UserSettingsPage.tsx';
import MarketplacePage from '@/modules/marketplace/presentation/ui/MarketplacePage.tsx';
import Sidebar from '@/modules/core/presentation/ui/Sidebar.tsx';
import { Layout } from '@/modules/core/presentation/ui/Layout.tsx';

//TODO: navigation and overlay
export const CoreRoot = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path='/login' element={<LoginPage />} />
        <Route element={<Layout />}>
          <Route path='/user_settings' element={<UserSettingsPage />} />
          <Route path='/marketplace' element={<MarketplacePage />} />
          <Route path='*' element={<Navigate to='/login' replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
};

export default CoreRoot;
