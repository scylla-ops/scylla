import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { KeyRound, Plus, Trash } from 'lucide-react';
import { Badge, Button, Card, CardContent } from '@shadcn';
import { Skeleton } from '@shadcn/skeleton.tsx';
import { Switch } from '@shadcn/switch.tsx';
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
import { FormDialog, SecretRevealDialog } from '@shared/presentation/ui';
import type { FormChange } from '@shared/presentation/structs/scylla-form.struct.ts';
import { formatDate } from '@shared/utils/date-utils.ts';
import { useAppSecrets } from '@/modules/features/apps/presentation/hooks/use-apps.ts';
import { createAppSecretItems } from '@/modules/features/apps/presentation/utils/create-app-secret-form-items.ts';
import type { AppEntity } from '@/modules/features/apps/domain/entities/app.entity.ts';
import type { CreatedAppSecret } from '@/modules/features/apps/domain/structs/app.struct.ts';

interface AppSecretsCardProps {
  app: AppEntity;
}

export const AppSecretsCard = ({ app }: AppSecretsCardProps) => {
  const { secrets, isLoading, createSecret, revokeSecret, setSecretEnabled } = useAppSecrets(
    app.id,
  );
  const [createOpen, setCreateOpen] = useState(false);
  const [created, setCreated] = useState<CreatedAppSecret | null>(null);
  const [toRevoke, setToRevoke] = useState<string | null>(null);

  const handleCreate = (values: FormChange[]) => {
    const label = values.find(v => v.id === 'label')?.value;
    if (!label?.trim()) return;
    createSecret.mutate(label.trim(), {
      onSuccess: data => {
        setCreateOpen(false);
        setCreated(data);
      },
    });
  };

  return (
    <Card>
      <CardContent className='p-5'>
        <div className='flex items-start justify-between gap-2'>
          <div>
            <h2 className='text-lg font-semibold'>
              <Trans>Secrets</Trans>
            </h2>
            <p className='text-sm text-muted-foreground'>
              <Trans>
                Credentials to authenticate this app. Disable cuts active sessions instantly; revoke
                deletes the secret.
              </Trans>
            </p>
          </div>
          <Button size='sm' onClick={() => setCreateOpen(true)}>
            <Plus className='mr-1 h-4 w-4' />
            <Trans>New secret</Trans>
          </Button>
        </div>

        <div className='mt-4 space-y-2'>
          {isLoading ? (
            Array.from({ length: 2 }).map((_, i) => (
              <Skeleton key={i} className='h-14 w-full rounded-md' />
            ))
          ) : secrets.length === 0 ? (
            <p className='rounded-md border border-dashed p-4 text-center text-sm text-muted-foreground'>
              <Trans>No secrets. Create one so this app can authenticate.</Trans>
            </p>
          ) : (
            secrets.map(secret => (
              <div
                key={secret.id}
                className='flex items-center justify-between gap-3 rounded-md border p-3'
              >
                <div className='flex min-w-0 items-center gap-3'>
                  <span className='flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-success/30 bg-success/10'>
                    <KeyRound className='h-4 w-4 text-success' />
                  </span>
                  <div className='min-w-0'>
                    <div className='flex items-center gap-2'>
                      <span className='truncate font-medium'>{secret.label}</span>
                      {secret.enabled ? (
                        <Badge className='bg-success/15 text-success hover:bg-success/15'>
                          <Trans>active</Trans>
                        </Badge>
                      ) : (
                        <Badge variant='secondary'>
                          <Trans>disabled</Trans>
                        </Badge>
                      )}
                    </div>
                    <p className='truncate font-mono text-xs text-muted-foreground'>
                      <Trans>created</Trans> {formatDate(secret.createdAt)}
                    </p>
                  </div>
                </div>
                <div className='flex shrink-0 items-center gap-3'>
                  <Switch
                    checked={secret.enabled}
                    disabled={setSecretEnabled.isPending}
                    onCheckedChange={enabled =>
                      setSecretEnabled.mutate({ secretId: secret.id, enabled })
                    }
                    aria-label='Toggle secret enabled'
                  />
                  <Button
                    variant='ghost'
                    size='icon'
                    className='h-8 w-8 text-muted-foreground hover:text-destructive'
                    onClick={() => setToRevoke(secret.id)}
                  >
                    <Trash className='h-4 w-4' />
                  </Button>
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>

      <FormDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        title={<Trans>New secret</Trans>}
        description={<Trans>Add a credential for this app.</Trans>}
        items={createAppSecretItems()}
        isPending={createSecret.isPending}
        submitLabel={<Trans>Create &amp; reveal secret →</Trans>}
        onSubmit={handleCreate}
      />

      {created && (
        <SecretRevealDialog
          open={!!created}
          entityKind='app'
          entity={{ id: app.id, name: `${app.name} · ${created.credential.label}` }}
          secret={created.secret}
          onClose={() => setCreated(null)}
        />
      )}

      <AlertDialog open={!!toRevoke} onOpenChange={o => !o && setToRevoke(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Revoke secret?</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              <Trans>
                This permanently deletes the secret and immediately cuts any session using it.
                Cannot be undone.
              </Trans>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (toRevoke) revokeSecret.mutate(toRevoke);
                setToRevoke(null);
              }}
            >
              <Trans>Revoke</Trans>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  );
};
