import { ScyllaBreadcrumbs } from '@/modules/layout/presentation/ui/ScyllaBreadcrumbs.tsx';
import ScyllaSidebarTrigger from '@/modules/layout/presentation/ui/ScyllaSidebarTrigger.tsx';

export const TopBar = () => {
  return (
    <header className={'flex flex-row p-2 items-center  gap-4'}>
      <ScyllaSidebarTrigger />
      <ScyllaBreadcrumbs />
    </header>
  );
};
