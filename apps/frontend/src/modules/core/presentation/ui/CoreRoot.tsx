import { BrowserRouter, Route, Navigate, Routes } from 'react-router-dom';
import LoginPage from '@/modules/login/presentation/ui/LoginPage.tsx';

import UserSettingsPage from '@/modules/user_settings/presentation/ui/UserSettingsPage.tsx';
import MarketplacePage from '@/modules/marketplace/presentation/ui/MarketplacePage.tsx';

//TODO: navigation and overlay
export const CoreRoot = () => {
  return (
    <BrowserRouter>
      <Routes>
        <Route path='/login' element={<LoginPage />} />
        <Route path='/user_settings' element={<UserSettingsPage />} />
        <Route path='/marketplace' element={<MarketplacePage />}></Route>
        <Route path='*' element={<Navigate to='/login' replace />} />
      </Routes>
    </BrowserRouter>
  );
};

export default CoreRoot;
