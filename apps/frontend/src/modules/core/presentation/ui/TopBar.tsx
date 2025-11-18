import { useMatches } from 'react-router-dom';
import type { ReactNode } from 'react';
import { SidebarTrigger } from '@shadcn/sidebar.tsx';

interface RouteHandle {
  topbar?: ReactNode;
}

//TODO: variable for the bg of the topbar
export const TopBar = () => {
  const matches = useMatches();

  const matchWithTopbar = matches.find(m => (m.handle as RouteHandle | undefined)?.topbar);

  const content = (matchWithTopbar?.handle as RouteHandle | undefined)?.topbar;

  return (
    <header
      className={
        'flex flex-row items-center justify-between px-4 py-2 bg-background border-b-2 h-12'
      }
    >
      <SidebarTrigger />
      {content}
    </header>
  );
};
