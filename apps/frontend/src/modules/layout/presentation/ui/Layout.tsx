import { AppSidebar } from '@/modules/layout/presentation/ui/AppSidebar.tsx';
import { SidebarInset, SidebarProvider } from '@/modules/shared/presentation/ui/shadcn/sidebar.tsx';
import { TopBar } from '@/modules/layout/presentation/ui/TopBar.tsx';
import { AnimatedOutlet } from '@/modules/shared/presentation/ui/layout/AnimatedOutlet.tsx';
import { NewTriggerFeaturePopup } from '@/modules/layout/presentation/ui/NewTriggerFeaturePopup.tsx';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { Trans } from '@lingui/react/macro';
import { useCreateOrganization } from '@/modules/features/organization/presentation/hooks/useCreateOrganization.ts';
import { ScyllaForm } from '@shared/presentation/ui/forms/ScyllaForm.tsx';
import { createOrganizationItems } from '@/modules/features/organization/presentation/utils/create-organization-form-items.ts';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/modules/shared/presentation/ui/shadcn/card.tsx';
import scyllaLogo from '@/assets/logo_scylla.png';
import { AnimatePresence, motion } from 'framer-motion';
import { useNavigate } from 'react-router-dom';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import { usePermissionSync } from '@/modules/features/permission/presentation/hooks/use-permission-sync.ts';

export const Layout = () => {
  const { organizations, isLoading } = useOrganizations();
  const createOrganization = useCreateOrganization();
  const navigate = useNavigate();
  const setOrganization = useContextStore(state => state.setOrganization);

  // One place loads the signed-in user's permissions into the context store
  // (at login, then on org/project switch); every `can()` reads from there.
  usePermissionSync();

  if (isLoading) {
    return (
      <div className='flex items-center justify-center h-screen w-screen bg-background'>
        <motion.img
          src={scyllaLogo}
          alt='Scylla'
          className='h-24 w-24'
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: [0, 1, 1, 0.5], scale: [0.8, 1, 1, 0.95] }}
          transition={{ duration: 2, repeat: Infinity, ease: 'easeInOut' }}
        />
      </div>
    );
  }

  //todo: "No organization / first connexion page, move from here"
  if (!organizations || organizations.length === 0) {
    return (
      <AnimatePresence mode='wait'>
        <motion.main
          key={location.pathname}
          initial={{ opacity: 0, scale: 0.95, y: 20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: -10 }}
          transition={{
            duration: 0.8,
            ease: [0.22, 1, 0.36, 1],
          }}
          className='flex flex-col h-full w-full p-2'
        >
          <div className='w-full h-full flex flex-col items-center min-h-screen bg-background'>
            <img src={scyllaLogo} alt='Scylla' className='h-1/6 w-1/6' />
            <Card className='w-full max-w-md'>
              <CardHeader className='text-center'>
                <CardTitle className='text-2xl'>
                  <Trans>Welcome to Scylla!</Trans>
                </CardTitle>
                <CardDescription>
                  <Trans>To get started, please create your first organization.</Trans>
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ScyllaForm
                  items={createOrganizationItems()}
                  onSubmit={values => {
                    const name = values.find(v => v.id === 'name')?.value;
                    const description = values.find(v => v.id === 'description')?.value;
                    if (name) {
                      createOrganization.mutate(
                        { name, description: description || '' },
                        {
                          onSuccess: data => {
                            const orgId = idValue(data?.organizationId);
                            setOrganization(orgId, name);
                            void navigate(`/${slugifyOrgName(name)}/users/me`);
                          },
                        },
                      );
                    }
                  }}
                  buttonLabel={<Trans>Create</Trans>}
                />
              </CardContent>
            </Card>
          </div>
        </motion.main>
      </AnimatePresence>
    );
  }

  return (
    <SidebarProvider className='w-screen h-screen'>
      <AppSidebar />
      <SidebarInset className='flex flex-col flex-1 min-w-0 border border-sidebar-border bg-background'>
        <TopBar />
        <div className='p-4 flex-1 min-h-0'>
          <AnimatedOutlet />
        </div>
      </SidebarInset>
      <NewTriggerFeaturePopup />
    </SidebarProvider>
  );
};
