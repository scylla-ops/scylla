import { useMatches } from 'react-router-dom';
import type { ReactNode } from 'react';

interface RouteHandle {
  topbar?: ReactNode;
}

export const TopBar = () => {
  const matches = useMatches();

  const matchWithTopbar = matches.find(m => (m.handle as RouteHandle | undefined)?.topbar);

  const content = (matchWithTopbar?.handle as RouteHandle | undefined)?.topbar;

  return <header style={{ padding: '12px', borderBottom: '1px solid #ddd' }}>{content}</header>;
};
