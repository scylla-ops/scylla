import { useMatches } from 'react-router-dom';
import { SidebarTrigger } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import type { RouteHandle } from '@core/presentation/models/RouteHandle.ts';

//TODO: variable for the bg of the topbar
export const TopBar = () => {
  const matches = useMatches();

  const matchWithTopbar = matches.find(m => (m.handle as RouteHandle | undefined)?.topbar);

  const content = (matchWithTopbar?.handle as RouteHandle | undefined)?.topbar;

  return (
    <header
      className={
        'flex flex-row items-center justify-between px-4 bg-background border-b-2 h-12 min-h-12'
      }
    >
      <SidebarTrigger />
      {content}
    </header>
  );
};
