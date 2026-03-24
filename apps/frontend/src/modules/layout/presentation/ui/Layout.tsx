import { AppSidebar } from '@/modules/layout/presentation/ui/AppSidebar.tsx';
import { useMatches } from 'react-router-dom';
import { SidebarInset, SidebarProvider } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { TopBar } from '@/modules/layout/presentation/ui/TopBar.tsx';
import type { RouteHandle } from '@core/presentation/models/RouteHandle.ts';
import { Tabs } from '@shadcn/tabs.tsx';
import { AnimatedOutlet } from '@/modules/shared/presentation/ui/AnimatedOutlet.tsx';

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
          <AnimatedOutlet />
        </Tabs>
      </SidebarInset>
    </SidebarProvider>
  );
};
