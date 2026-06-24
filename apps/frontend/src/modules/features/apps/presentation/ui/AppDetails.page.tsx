import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useApp, useApps } from '@/modules/features/apps/presentation/hooks/use-apps.ts';
import { AppSecretsCard } from '@/modules/features/apps/presentation/ui/components/AppSecretsCard.tsx';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { useResourceError } from '@shared/presentation/hooks/use-resource-error.ts';
import { Badge, Button, Card, CardContent } from '@shadcn';
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
import { CodeSnippet } from '@shadcn/code-snippet.tsx';
import { Switch } from '@shadcn/switch.tsx';
import { KeyRound } from 'lucide-react';
import { formatDate } from '@shared/utils/date-utils.ts';
import { Trans } from '@lingui/react/macro';

const Label = ({ children }: { children: React.ReactNode }) => (
  <dt className='font-mono text-[11px] uppercase tracking-wide text-muted-foreground'>{children}</dt>
);

export const AppDetailsPage = () => {
  const { appId } = useParams<{ appId: string }>();
  const navigate = useNavigate();
  const { data: app, isLoading, isError, error } = useApp(appId ?? '');
  const { deleteApp, setAppActive } = useApps();
  const [confirmDelete, setConfirmDelete] = useState(false);

  // NOT_FOUND (deleted / bad id) → toast + back to the apps list.
  const { redirecting } = useResourceError({
    error,
    redirectTo: '..',
    notFoundMessage: 'App not found',
  });

  if (redirecting) return null;
  if (isLoading) return <Skeleton className='m-4 h-64 rounded-xl' />;
  if (isError || !app) return <ErrorState message='Error loading app' />;

  return (
    <div className='space-y-4 p-4'>
      <div className='flex items-start justify-between gap-3'>
        <div className='flex items-center gap-3'>
          <span className='flex h-12 w-12 items-center justify-center rounded-lg border border-success/30 bg-success/10'>
            <KeyRound className='h-6 w-6 text-success' />
          </span>
          <div>
            <div className='flex items-center gap-2'>
              <h1 className='text-xl font-semibold'>{app.name}</h1>
              <Badge variant={app.isActive ? 'default' : 'secondary'}>
                {app.isActive ? <Trans>active</Trans> : <Trans>inactive</Trans>}
              </Badge>
            </div>
            <p className='text-sm text-muted-foreground'>
              <Trans>Machine credential · no runtime presence</Trans>
            </p>
          </div>
        </div>
        <div className='flex items-center gap-3'>
          <label className='flex items-center gap-2 text-sm text-muted-foreground'>
            <Switch
              checked={app.isActive}
              disabled={setAppActive.isPending}
              onCheckedChange={active => setAppActive.mutate({ appId: app.id, active })}
              aria-label='Toggle app active'
            />
            {app.isActive ? <Trans>active</Trans> : <Trans>inactive</Trans>}
          </label>
          <Button variant='outline' disabled title='Coming soon'>
            <Trans>Rename</Trans>
          </Button>
          <Button variant='destructive' onClick={() => setConfirmDelete(true)}>
            <Trans>Delete</Trans>
          </Button>
        </div>
      </div>

      <Card>
        <CardContent className='p-5'>
          <dl className='grid gap-4 sm:grid-cols-2'>
            <div className='sm:col-span-2'>
              <dd className='mt-1'>
                <CodeSnippet
                  multiline
                  value={app.id}
                  label={<Trans>App ID</Trans>}
                  copyToast='App id copied'
                />
              </dd>
            </div>
            <div>
              <Label>
                <Trans>Name</Trans>
              </Label>
              <dd className='mt-1'>{app.name}</dd>
            </div>
            <div>
              <Label>
                <Trans>Status</Trans>
              </Label>
              <dd className='mt-1'>
                <Badge variant={app.isActive ? 'default' : 'secondary'}>
                  {app.isActive ? <Trans>active</Trans> : <Trans>inactive</Trans>}
                </Badge>
              </dd>
            </div>
            <div>
              <Label>
                <Trans>Created</Trans>
              </Label>
              <dd className='mt-1'>{formatDate(app.createdAt)}</dd>
            </div>
            <div>
              <Label>
                <Trans>Updated</Trans>
              </Label>
              <dd className='mt-1'>{formatDate(app.updatedAt)}</dd>
            </div>
          </dl>

          <div className='mt-5 rounded-md border-l-2 border-success bg-success/5 p-3 text-sm text-muted-foreground'>
            <Trans>
              Apps are credentials, not processes. There's nothing to monitor here — no
              online/offline, no runs. If you need observability, create an Agent instead.
            </Trans>
          </div>
        </CardContent>
      </Card>

      <AppSecretsCard app={app} />

      <AlertDialog open={confirmDelete} onOpenChange={setConfirmDelete}>
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
                deleteApp.mutate(app.id, { onSuccess: () => void navigate('..') });
                setConfirmDelete(false);
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
