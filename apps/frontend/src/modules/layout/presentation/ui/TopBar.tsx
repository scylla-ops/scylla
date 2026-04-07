import { useMatches } from 'react-router-dom';
import { SidebarTrigger } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import type { RouteHandle } from '@core/presentation/models/RouteHandle.ts';
import { ScyllaBreadcrumbs } from '@/modules/layout/presentation/ui/ScyllaBreadcrumbs.tsx';

//TODO: variable for the bg of the topbar
export const TopBar = () => {
  const matches = useMatches();

  const matchWithTopbar = matches.find(m => (m.handle as RouteHandle | undefined)?.topbar);

  const content = (matchWithTopbar?.handle as RouteHandle | undefined)?.topbar;

  return (
    <header className={'flex flex-col items-center justify-between p-8 h-28 min-h-28 gap-4'}>
      <div className={'flex-1  w-full align-middle items-center flex flex-row gap-4'}>
        <SidebarTrigger />
        <ScyllaBreadcrumbs />
      </div>
      <div className={'flex-1 w-full'}>{content}</div>
    </header>
  );
};
