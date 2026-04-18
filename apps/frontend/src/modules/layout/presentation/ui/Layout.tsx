import { AppSidebar } from '@/modules/layout/presentation/ui/AppSidebar.tsx';
import { SidebarInset, SidebarProvider } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { TopBar } from '@/modules/layout/presentation/ui/TopBar.tsx';
import { AnimatedOutlet } from '@/modules/shared/presentation/ui/AnimatedOutlet.tsx';

export const Layout = () => {
  return (
    <SidebarProvider className='w-screen h-screen overflow-hidden'>
      <AppSidebar />
      <SidebarInset className='flex flex-col flex-1 min-w-0 border border-sidebar-border bg-background overflow-hidden'>
        <TopBar />
        <div className='flex-1 min-h-0 overflow-hidden'>
          <AnimatedOutlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};
