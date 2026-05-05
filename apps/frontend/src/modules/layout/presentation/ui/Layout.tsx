import { AppSidebar } from '@/modules/layout/presentation/ui/AppSidebar.tsx';
import { SidebarInset, SidebarProvider } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { TopBar } from '@/modules/layout/presentation/ui/TopBar.tsx';
import { AnimatedOutlet } from '@/modules/shared/presentation/ui/AnimatedOutlet.tsx';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { AddOrganizationDialog } from '@/modules/features/organization/presentation/ui/AddOrganizationDialog.tsx';
import { Trans } from '@lingui/react/macro';

export const Layout = () => {
  const { organizations, isLoading } = useOrganizations();

  // If not loading and no organizations, force creation
  if (!isLoading && (!organizations || organizations.length === 0)) {
    return (
      <div className='flex items-center justify-center min-h-screen bg-background'>
        <div className='text-center'>
          <h1 className='text-2xl font-bold mb-4'>
            <Trans>Welcome to Scylla!</Trans>
          </h1>
          <p className='text-muted-foreground mb-6'>
            <Trans>To get started, please create your first organization.</Trans>
          </p>
          <AddOrganizationDialog
            open={true}
            setOpen={() => {}} // Prevent closing
            hideCancel={true}
          />
        </div>
      </div>
    );
  }

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
