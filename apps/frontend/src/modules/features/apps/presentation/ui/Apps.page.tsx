import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useApps } from '@/modules/features/apps/presentation/hooks/use-apps.ts';
import { createAppItems } from '@/modules/features/apps/presentation/utils/create-app-form-items.ts';
import { AppCard } from '@/modules/features/apps/presentation/ui/components/AppCard.tsx';
import type { CreatedApp } from '@/modules/features/apps/domain/structs/app.struct.ts';
import { FeatureHeader, FormDialog, SecretRevealDialog } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { Button, Card } from '@shadcn';
import { Skeleton } from '@shadcn/skeleton.tsx';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@shadcn/alert-dialog.tsx';
import type { FormChange } from '@shared/presentation/structs/scylla-form.struct.ts';
import { KeyRound, Plus } from 'lucide-react';
import { Trans } from '@lingui/react/macro';

export const AppsPage = () => {
  const { apps, isLoading, isError, createApp, deleteApp } = useApps();
  const navigate = useNavigate();
  const [createOpen, setCreateOpen] = useState(false);
  const [created, setCreated] = useState<CreatedApp | null>(null);
  const [toDelete, setToDelete] = useState<string | null>(null);

  const handleCreate = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value;
    if (!name?.trim()) return;
    createApp.mutate(name.trim(), {
      onSuccess: data => {
        setCreateOpen(false);
        setCreated(data);
      },
    });
  };

  const activeCount = apps.filter(a => a.isActive).length;

  if (isError) return <ErrorState message='Error loading apps' />;

  return (
    <div className='flex flex-col w-full h-full overflow-hidden'>
      <div className='px-2 pt-2'>
        <p className='mb-1 font-mono text-xs uppercase tracking-wider text-muted-foreground'>
          <Trans>Service</Trans>
        </p>
        <FeatureHeader count={apps.length} label='App' onNew={() => setCreateOpen(true)} />
        {!isLoading && (
          <p className='mt-1 font-mono text-xs text-muted-foreground'>
            {activeCount} <Trans>active</Trans> · {apps.length - activeCount}{' '}
            <Trans>inactive</Trans>
          </p>
        )}
      </div>

      {isLoading ? (
        <div className='grid gap-3 p-2 sm:grid-cols-2 lg:grid-cols-3'>
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className='h-32 w-full rounded-xl' />
          ))}
        </div>
      ) : apps.length === 0 ? (
        <div className='flex flex-1 items-center justify-center p-6'>
          <Card className='flex max-w-sm flex-col items-center gap-3 p-8 text-center'>
            <span className='flex h-14 w-14 items-center justify-center rounded-xl border border-dashed border-success/40 bg-success/5'>
              <KeyRound className='h-6 w-6 text-success' />
            </span>
            <h2 className='text-lg font-semibold'>
              <Trans>No apps yet</Trans>
            </h2>
            <p className='text-sm text-muted-foreground'>
              <Trans>
                Apps are machine credentials. Create one to let an automation authenticate against
                the Scylla API.
              </Trans>
            </p>
            <Button onClick={() => setCreateOpen(true)}>
              <Trans>Create your first app</Trans>
            </Button>
          </Card>
        </div>
      ) : (
        <div className='grid gap-3 p-2 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3'>
          {apps.map(app => (
            <AppCard key={app.id} app={app} onRequestDelete={setToDelete} />
          ))}
          <button
            type='button'
            onClick={() => setCreateOpen(true)}
            className='flex min-h-32 items-center justify-center gap-2 rounded-xl border border-dashed text-sm text-muted-foreground transition-colors hover:bg-accent/50 hover:text-foreground'
          >
            <Plus className='h-4 w-4' />
            <Trans>New App</Trans>
          </button>
        </div>
      )}

      <FormDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        title={<Trans>New App</Trans>}
        description={<Trans>Get credentials to authenticate.</Trans>}
        items={createAppItems()}
        isPending={createApp.isPending}
        submitLabel={<Trans>Create &amp; reveal secret →</Trans>}
        onSubmit={handleCreate}
      />

      {created && (
        <SecretRevealDialog
          open={!!created}
          entityKind='app'
          entity={{ id: created.app.id, name: created.app.name }}
          secret={created.secret}
          onClose={() => {
            const id = created.app.id;
            setCreated(null);
            void navigate(id);
          }}
        />
      )}

      <AlertDialog open={!!toDelete} onOpenChange={o => !o && setToDelete(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Delete app?</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              <Trans>This revokes the app's credentials. Cannot be undone.</Trans>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (toDelete) deleteApp.mutate(toDelete);
                setToDelete(null);
              }}
            >
              <Trans>Delete</Trans>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};
