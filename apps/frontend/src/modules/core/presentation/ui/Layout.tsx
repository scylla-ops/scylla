import { AppSidebar } from '@core/presentation/ui/AppSidebar.tsx';
import { Outlet } from 'react-router-dom';
import { SidebarProvider, SidebarTrigger } from '@shadcn/sidebar.tsx';
import { TopBar } from '@core/presentation/ui/TopBar.tsx';

export const Layout = () => (
  <SidebarProvider>
    <AppSidebar />
    <main className={'w-full h-full'}>
      <SidebarTrigger />
      <TopBar />
      <Outlet />
    </main>
  </SidebarProvider>
);
