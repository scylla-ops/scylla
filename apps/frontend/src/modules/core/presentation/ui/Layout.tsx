import { AppSidebar } from '@core/presentation/ui/AppSidebar.tsx';
import { Outlet } from 'react-router-dom';
import { SidebarProvider } from '@shadcn/sidebar.tsx';
import { TopBar } from '@core/presentation/ui/TopBar.tsx';

export const Layout = () => (
  <SidebarProvider>
    <AppSidebar />
    <div className={'h-full w-full'}>
      <TopBar />
      <main className={'w-full h-full p-2'}>
        <Outlet />
      </main>
    </div>
  </SidebarProvider>
);
