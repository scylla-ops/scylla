import { SidebarTrigger } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { ScyllaBreadcrumbs } from '@/modules/layout/presentation/ui/ScyllaBreadcrumbs.tsx';

//TODO: variable for the bg of the topbar
export const TopBar = () => {
  return (
    <header className={'flex flex-row p-2 items-center  gap-4'}>
      <SidebarTrigger />
      <ScyllaBreadcrumbs />
    </header>
  );
};
