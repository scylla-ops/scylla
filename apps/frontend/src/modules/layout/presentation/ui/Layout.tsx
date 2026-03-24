import { AppSidebar } from '@/modules/layout/presentation/ui/AppSidebar.tsx';
import { Outlet, useMatches } from 'react-router-dom';
import { SidebarInset, SidebarProvider } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { TopBar } from '@/modules/layout/presentation/ui/TopBar.tsx';
import type { RouteHandle } from '@core/presentation/models/RouteHandle.ts';
import { Tabs } from '@shadcn/tabs.tsx';

export const Layout = () => {
  const matches = useMatches();
  const matchWithTabsDefaultValue = matches.find(
    m => (m.handle as RouteHandle | undefined)?.tabsDefaultValue,
  );
  const tabsDefaultValue = (matchWithTabsDefaultValue?.handle as RouteHandle | undefined)
    ?.tabsDefaultValue;

  return (
    <SidebarProvider className={'h-screen w-screen'}>
      <AppSidebar />
      <SidebarInset className={'border border-sidebar-border bg-background'}>
        <Tabs key={tabsDefaultValue ?? 'no-tabs'} defaultValue={tabsDefaultValue}>
          <TopBar />
          <main className={'h-full w-full p-2'}>
            <Outlet />
          </main>
        </Tabs>
      </SidebarInset>
    </SidebarProvider>
  );
};
