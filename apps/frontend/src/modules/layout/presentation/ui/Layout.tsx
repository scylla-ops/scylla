import { AppSidebar } from '@/modules/layout/presentation/ui/AppSidebar.tsx';
import { Outlet, useMatches } from 'react-router-dom';
import { SidebarProvider } from '@shadcn/sidebar.tsx';
import { TopBar } from '@/modules/layout/presentation/ui/TopBar.tsx';
import { Tabs } from '@shadcn/tabs.tsx';
import type { RouteHandle } from '@core/presentation/models/RouteHandle.ts';

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
      <Tabs
        key={tabsDefaultValue ?? 'no-tabs'}
        defaultValue={tabsDefaultValue}
        className={'h-full w-full gap-0'}
      >
        <TopBar />
        <main className={'h-full w-full p-2'}>
          <Outlet />
        </main>
      </Tabs>
    </SidebarProvider>
  );
};
