<script lang="ts">
  import KeyRoundIcon from '@lucide/svelte/icons/key-round';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import TrashIcon from '@lucide/svelte/icons/trash';
  import { createMutation, createQuery } from '@platform/query';
  import { Badge, Button, Card, CardContent, Skeleton, Switch } from '@shadcn';
  import {
    ConfirmOperationAlertDialog,
    FormDialog,
    SecretRevealDialog,
    type FormValues,
  } from '@shared/presentation/ui';
  import { formatDate } from '@shared/utils/date-utils.ts';
  import { activeLocale, t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { AppEntity } from '../../../domain/entities/app.entity.ts';
  import type { CreatedAppSecret } from '../../../domain/structs/app.struct.ts';
  import { appMutations, appQueries } from '../../apps.queries.ts';
  import { createAppSecretItems } from '../../utils/create-app-secret-form-items.ts';
  import { appsMessages } from '../apps.messages.ts';

  interface Props {
    app: AppEntity;
  }

  let { app }: Props = $props();

  const secretsQuery = createQuery(() => appQueries.secretsOf(app.id));
  const createSecret = createMutation(() => appMutations.createSecret(app.id));
  const revokeSecret = createMutation(() => appMutations.revokeSecret(app.id));
  const setSecretEnabled = createMutation(() => appMutations.setSecretEnabled(app.id));

  const secrets = $derived(secretsQuery.data ?? []);

  let createOpen = $state(false);
  let created = $state<CreatedAppSecret | null>(null);
  let pendingRevocation = $state<string | null>(null);

  // Reading the locale rebuilds the labels on a switch.
  const items = $derived((activeLocale(), createAppSecretItems()));

  const handleCreate = ({ label }: FormValues<'label'>) => {
    if (!label.trim()) return;

    createSecret.mutate(label.trim(), {
      onSuccess: data => {
        createOpen = false;
        created = data;
      },
    });
  };

  const confirmRevoke = () => {
    if (pendingRevocation) revokeSecret.mutate(pendingRevocation);
    pendingRevocation = null;
  };
</script>

<Card>
  <CardContent class="p-5">
    <div class="flex items-start justify-between gap-2">
      <div>
        <h2 class="text-lg font-semibold">{t(appsMessages.secrets)}</h2>
        <p class="text-sm text-muted-foreground">{t(appsMessages.secretsBody)}</p>
      </div>
      <Button size="sm" onclick={() => (createOpen = true)}>
        <PlusIcon class="mr-1 h-4 w-4" />
        {t(appsMessages.newSecret)}
      </Button>
    </div>

    <div class="mt-4 space-y-2">
      {#if secretsQuery.isLoading}
        {#each { length: 2 } as _, index (index)}
          <Skeleton class="h-14 w-full rounded-md" />
        {/each}
      {:else if secrets.length === 0}
        <p
          class="rounded-md border border-dashed p-4 text-center text-sm text-muted-foreground"
        >
          {t(appsMessages.noSecrets)}
        </p>
      {:else}
        {#each secrets as secret (secret.id)}
          <div class="flex items-center justify-between gap-3 rounded-md border p-3">
            <div class="flex min-w-0 items-center gap-3">
              <span
                class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-success/30 bg-success/10"
              >
                <KeyRoundIcon class="h-4 w-4 text-success" />
              </span>
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <span class="truncate font-medium">{secret.label}</span>
                  {#if secret.enabled}
                    <Badge class="bg-success/15 text-success hover:bg-success/15">
                      {t(appsMessages.active)}
                    </Badge>
                  {:else}
                    <Badge variant="secondary">{t(appsMessages.disabled)}</Badge>
                  {/if}
                </div>
                <p class="truncate font-mono text-xs text-muted-foreground">
                  {t(appsMessages.created)}
                  {formatDate(secret.createdAt)}
                </p>
              </div>
            </div>
            <div class="flex shrink-0 items-center gap-3">
              <Switch
                checked={secret.enabled}
                disabled={setSecretEnabled.isPending}
                onCheckedChange={enabled =>
                  setSecretEnabled.mutate({ secretId: secret.id, enabled })}
                aria-label={t(appsMessages.toggleSecretEnabled)}
              />
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 text-muted-foreground hover:text-destructive"
                onclick={() => (pendingRevocation = secret.id)}
              >
                <TrashIcon class="h-4 w-4" />
                <span class="sr-only">{t(appsMessages.revokeSecretAction)}</span>
              </Button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </CardContent>
</Card>

<FormDialog
  open={createOpen}
  onOpenChange={open => (createOpen = open)}
  title={t(appsMessages.newSecret)}
  description={t(appsMessages.addCredential)}
  {items}
  isPending={createSecret.isPending}
  submitLabel={t(appsMessages.createAndReveal)}
  onSubmit={handleCreate}
/>

{#if created}
  <SecretRevealDialog
    open
    title={t(appsMessages.secretCredentialsOf(app.name, created.credential.label))}
    description={t(appsMessages.copySecretOnce)}
    secret={created.secret}
    secretLabel={t(appsMessages.secret)}
    revealedNote={t(appsMessages.useTheseCredentials)}
    onClose={() => (created = null)}
  />
{/if}

<ConfirmOperationAlertDialog
  open={pendingRevocation !== null}
  onOpenChange={open => {
    if (!open) pendingRevocation = null;
  }}
  onContinue={confirmRevoke}
  title={t(appsMessages.revokeSecretTitle)}
  description={t(appsMessages.revokeSecretBody)}
/>
