import { AppSidebar } from '@/modules/layout/presentation/ui/AppSidebar.tsx';
import { SidebarInset, SidebarProvider } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { TopBar } from '@/modules/layout/presentation/ui/TopBar.tsx';
import { AnimatedOutlet } from '@/modules/shared/presentation/ui/AnimatedOutlet.tsx';

export const Layout = () => {
  return (
    <SidebarProvider className={'h-screen w-screen'}>
      <AppSidebar />
      <SidebarInset className={'flex flex-col border border-sidebar-border bg-background gap-2'}>
        <TopBar />
        <div className={'h-full w-full p-2'}>
          <AnimatedOutlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};
