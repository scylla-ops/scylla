import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';
import { KeyRound, Plus, Trash } from 'lucide-react';
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
import { BackButton, FeatureHeader, FormDialog } from '@shared/presentation/ui';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import type { FormChange } from '@shared/presentation/models/scylla-form.model.ts';
import { formatDate } from '@shared/utils/date-utils.ts';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import {
  useCreateSecret,
  useDeleteSecret,
  useSecrets,
} from '@/modules/features/secret/presentation/hooks/use-secrets.ts';
import { createSecretItems } from '@/modules/features/secret/presentation/utils/create-secret-form-items.ts';

export const SecretsPage = () => {
  const { projectId } = useParams();
  const { goBack } = useScyllaNavigate();
  const { secrets, isLoading, isError } = useSecrets(projectId ?? '');
  const createSecret = useCreateSecret(projectId ?? '');
  const deleteSecret = useDeleteSecret(projectId ?? '');

  const [createOpen, setCreateOpen] = useState(false);
  const [toDelete, setToDelete] = useState<string | null>(null);

  const handleCreate = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value?.trim();
    const value = values.find(v => v.id === 'value')?.value ?? '';
    const description = values.find(v => v.id === 'description')?.value?.trim() ?? '';
    if (!name || !value) return;
    createSecret.mutate({ name, value, description }, { onSuccess: () => setCreateOpen(false) });
  };

  if (isError) return <ErrorState message='Unable to load secrets' />;

  return (
    <div className='flex flex-col gap-4 w-full h-full p-4'>
      <div className='flex items-center gap-4 w-full'>
        <BackButton iconOnly onClick={() => goBack()} />
        <FeatureHeader
          count={secrets.length}
          label='Secret'
          onNew={() => setCreateOpen(true)}
          newLabel={<Trans>New secret</Trans>}
        />
      </div>

      <p className='text-sm text-muted-foreground'>
        <Trans>
          Project secrets are encrypted at rest and referenced by name from pipeline step
          environment variables. Their values are write-only and never displayed.
        </Trans>
      </p>

      <div className='flex-1 min-h-0 overflow-auto'>
        {isLoading ? (
          <div className='space-y-2'>
            {Array.from({ length: 3 }).map((_, i) => (
              <Skeleton key={i} className='h-16 w-full rounded-md' />
            ))}
          </div>
        ) : secrets.length === 0 ? (
          <div className='flex items-center justify-center h-full min-h-80'>
            <Card className='flex max-w-sm flex-col items-center gap-3 p-8 text-center'>
              <span className='flex h-14 w-14 items-center justify-center rounded-xl border border-dashed border-primary/40 bg-primary/5'>
                <KeyRound className='h-6 w-6 text-primary' />
              </span>
              <h2 className='text-lg font-semibold'>
                <Trans>No secrets yet</Trans>
              </h2>
              <p className='text-sm text-muted-foreground'>
                <Trans>
                  Create a secret to reference it from a pipeline step's environment variables.
                </Trans>
              </p>
              <Button onClick={() => setCreateOpen(true)}>
                <Plus className='mr-1 h-4 w-4' />
                <Trans>Create your first secret</Trans>
              </Button>
            </Card>
          </div>
        ) : (
          <div className='space-y-2'>
            {secrets.map(secret => (
              <div
                key={secret.id}
                className='flex items-center justify-between gap-3 rounded-md border p-3'
              >
                <div className='flex min-w-0 items-center gap-3'>
                  <span className='flex h-9 w-9 shrink-0 items-center justify-center rounded-md border border-primary/30 bg-primary/10'>
                    <KeyRound className='h-4 w-4 text-primary' />
                  </span>
                  <div className='min-w-0'>
                    <span className='block truncate font-mono font-medium'>{secret.name}</span>
                    {secret.description && (
                      <p className='truncate text-sm text-muted-foreground'>{secret.description}</p>
                    )}
                    <p className='truncate font-mono text-xs text-muted-foreground'>
                      <Trans>created</Trans> {formatDate(secret.createdAt)}
                    </p>
                  </div>
                </div>
                <Button
                  variant='ghost'
                  size='icon'
                  className='h-8 w-8 shrink-0 text-muted-foreground hover:text-destructive'
                  onClick={() => setToDelete(secret.id)}
                  aria-label='Delete secret'
                >
                  <Trash className='h-4 w-4' />
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>

      <FormDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        title={<Trans>Create secret</Trans>}
        description={<Trans>The value is write-only — it is never shown again after saving.</Trans>}
        items={createSecretItems()}
        isPending={createSecret.isPending}
        submitLabel={<Trans>Create secret</Trans>}
        onSubmit={handleCreate}
      />

      <AlertDialog open={!!toDelete} onOpenChange={o => !o && setToDelete(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Delete secret?</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              <Trans>
                Any pipeline step referencing this secret will fail to resolve it. Cannot be undone.
              </Trans>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (toDelete) deleteSecret.mutate(toDelete);
                setToDelete(null);
              }}
            >
              <Trans>Delete secret</Trans>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};
