import { useState } from 'react';
import { useApps } from '@/modules/features/apps/presentation/hooks/use-apps.ts';
import { createAppItems } from '@/modules/features/apps/presentation/utils/create-app-form-items.ts';
import type { CreatedApp } from '@/modules/features/apps/domain/models/app.model.ts';
import { FeatureHeader, FormDialog } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { Button, Card, CardContent } from '@shadcn';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@shadcn/dialog.tsx';
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
import type { FormChange } from '@shared/presentation/models/scylla-form.model.ts';
import { formatDate } from '@shared/utils/date-utils.ts';
import { Trans } from '@lingui/react/macro';
import { toast } from 'sonner';

export const AppsPage = () => {
  const { apps, isLoading, isError, createApp, deleteApp } = useApps();
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

  const runCommand = created
    ? `scylla-agent --control-plane-url <CONTROL_PLANE_URL> --app-id ${created.app.id} --app-secret ${created.secret}`
    : '';

  const copy = async (text: string) => {
    await navigator.clipboard.writeText(text);
    toast.success('Copied to clipboard');
  };

  if (isLoading) return <></>;
  if (isError) return <ErrorState message='Error loading apps' />;

  return (
    <div className='flex flex-col w-full h-full overflow-hidden'>
      <div className='px-2 pt-2'>
        <FeatureHeader count={apps.length} label='App' onNew={() => setCreateOpen(true)} />
      </div>

      <div className='grid gap-3 p-2 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3'>
        {apps.map(app => (
          <Card key={app.id}>
            <CardContent className='p-4'>
              <div className='flex items-center justify-between gap-2'>
                <p className='font-semibold truncate'>{app.name}</p>
              </div>
              <p className='mt-1 font-mono text-xs text-muted-foreground break-all'>{app.id}</p>
              <p className='mt-2 text-xs text-muted-foreground'>
                <Trans>Created</Trans> {formatDate(app.createdAt)}
              </p>
              <div className='mt-3 flex justify-end'>
                <Button variant='outline' size='sm' onClick={() => setToDelete(app.id)}>
                  <Trans>Delete</Trans>
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
        {apps.length === 0 && (
          <p className='p-4 text-sm text-muted-foreground'>
            <Trans>No apps yet. Create one to connect an agent.</Trans>
          </p>
        )}
      </div>

      <FormDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        title={<Trans>Create a new app</Trans>}
        description={
          <Trans>
            An app is the machine identity an agent connects with. You will get a one-time secret.
          </Trans>
        }
        items={createAppItems()}
        isPending={createApp.isPending}
        submitLabel={<Trans>Create App</Trans>}
        onSubmit={handleCreate}
      />

      <Dialog open={!!created} onOpenChange={o => !o && setCreated(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>App created</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Copy the secret now — it is shown once and cannot be retrieved later.</Trans>
            </DialogDescription>
          </DialogHeader>
          {created && (
            <div className='space-y-3'>
              <div>
                <p className='text-xs text-muted-foreground'>
                  <Trans>Secret</Trans>
                </p>
                <div className='flex items-center gap-2'>
                  <code className='flex-1 rounded bg-muted p-2 text-xs break-all'>
                    {created.secret}
                  </code>
                  <Button size='sm' variant='outline' onClick={() => copy(created.secret)}>
                    <Trans>Copy</Trans>
                  </Button>
                </div>
              </div>
              <div>
                <p className='text-xs text-muted-foreground'>
                  <Trans>Run the agent</Trans>
                </p>
                <div className='flex items-center gap-2'>
                  <code className='flex-1 rounded bg-muted p-2 text-xs break-all'>
                    {runCommand}
                  </code>
                  <Button size='sm' variant='outline' onClick={() => copy(runCommand)}>
                    <Trans>Copy</Trans>
                  </Button>
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button onClick={() => setCreated(null)}>
              <Trans>Done</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog open={!!toDelete} onOpenChange={o => !o && setToDelete(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Delete app?</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              <Trans>
                This revokes the app's grants and disconnects its agent. Cannot be undone.
              </Trans>
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
