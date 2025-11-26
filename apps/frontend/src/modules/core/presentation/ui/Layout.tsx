import { AppSidebar } from '@core/presentation/ui/AppSidebar.tsx';
import { Outlet } from 'react-router-dom';
import { SidebarProvider } from '@shadcn/sidebar.tsx';
import { TopBar } from '@core/presentation/ui/TopBar.tsx';

export const Layout = () => (
  <SidebarProvider className={'h-screen w-screen'}>
    <AppSidebar />
    <div className={'h-full w-full flex flex-col'}>
      <TopBar />
      <main className={'h-full w-full p-2'}>
        <Outlet />
      </main>
    </div>
  </SidebarProvider>
);
