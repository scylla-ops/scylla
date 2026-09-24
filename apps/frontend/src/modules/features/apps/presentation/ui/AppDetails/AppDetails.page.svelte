<script lang="ts">
  import type { Snippet } from 'svelte';
  import KeyRoundIcon from '@lucide/svelte/icons/key-round';
  import { i18n } from '@lingui/core';
  import { createResourceError, scyllaNavigate, contextStore } from '@platform/context';
  import { createMutation, createQuery } from '@platform/query';
  import { Badge, Button, Card, CardContent, CodeSnippet, Skeleton, Switch } from '@shadcn';
  import { ConfirmOperationAlertDialog, ErrorState } from '@shared/presentation/ui';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { formatDate } from '@shared/utils/date-utils.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { appMutations, appQueries } from '../../apps.queries.ts';
  import AppSecretsCard from '../components/AppSecretsCard.svelte';
  import { appsMessages } from '../apps.messages.ts';

  interface Props {
    /** From the route. */
    appId?: string;
  }

  let { appId }: Props = $props();

  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id ?? '');

  const appQuery = createQuery(() => appQueries.byId(appId ?? ''));
  const deleteApp = createMutation(() => appMutations.remove(organizationId));
  const setAppActive = createMutation(() => appMutations.setActive(organizationId));

  const app = $derived(appQuery.data);

  // NOT_FOUND (deleted / bad id) → toast + back to the apps list.
  const resourceError = createResourceError({
    error: () => appQuery.error,
    redirectTo: '..',
    notFoundMessage: t(appsMessages.notFound),
  });

  let confirmDelete = $state(false);

  const handleDelete = () => {
    if (!app) return;
    deleteApp.mutate(app.id, { onSuccess: () => scyllaNavigate.navigate('..') });
    confirmDelete = false;
  };
</script>

{#snippet label(children: Snippet)}
  <dt class="font-mono text-[11px] uppercase tracking-wide text-muted-foreground">
    {@render children()}
  </dt>
{/snippet}

{#snippet statusBadge(isActive: boolean)}
  <Badge variant={isActive ? 'default' : 'secondary'}>
    {isActive ? t(appsMessages.active) : t(appsMessages.inactive)}
  </Badge>
{/snippet}

{#if resourceError.redirecting}
  <!-- Nothing: the redirect is already under way. -->
{:else if appQuery.isLoading}
  <Skeleton class="m-4 h-64 rounded-xl" />
{:else if appQuery.isError || !app}
  <ErrorState message={t(appsMessages.detailsLoadError)} />
{:else}
  <div class="space-y-4 p-4">
    <div class="flex items-start justify-between gap-3">
      <div class="flex items-center gap-3">
        <span
          class="flex h-12 w-12 items-center justify-center rounded-lg border border-success/30 bg-success/10"
        >
          <KeyRoundIcon class="h-6 w-6 text-success" />
        </span>
        <div>
          <div class="flex items-center gap-2">
            <h1 class="text-xl font-semibold">{app.name}</h1>
            {@render statusBadge(app.isActive)}
          </div>
          <p class="text-sm text-muted-foreground">{t(appsMessages.machineCredential)}</p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <label class="flex items-center gap-2 text-sm text-muted-foreground">
          <Switch
            checked={app.isActive}
            disabled={setAppActive.isPending}
            onCheckedChange={active => setAppActive.mutate({ appId: app.id, active })}
            aria-label={t(appsMessages.toggleActive)}
          />
          {app.isActive ? t(appsMessages.active) : t(appsMessages.inactive)}
        </label>
        <Button variant="outline" disabled title={t(appsMessages.comingSoon)}>
          {t(appsMessages.rename)}
        </Button>
        <Button variant="destructive" onclick={() => (confirmDelete = true)}>
          {t(appsMessages.delete)}
        </Button>
      </div>
    </div>

    <Card>
      <CardContent class="p-5">
        <dl class="grid gap-4 sm:grid-cols-2">
          <div class="sm:col-span-2">
            <dd class="mt-1">
              <CodeSnippet
                multiline
                value={app.id}
                label={appIdLabel}
                copyToast={i18n._(ToastMessages.APP_ID_COPIED)}
              />
            </dd>
          </div>
          <div>
            {@render label(nameLabel)}
            <dd class="mt-1">{app.name}</dd>
          </div>
          <div>
            {@render label(statusLabel)}
            <dd class="mt-1">{@render statusBadge(app.isActive)}</dd>
          </div>
          <div>
            {@render label(createdLabel)}
            <dd class="mt-1">{formatDate(app.createdAt)}</dd>
          </div>
          <div>
            {@render label(updatedLabel)}
            <dd class="mt-1">{formatDate(app.updatedAt)}</dd>
          </div>
        </dl>

        <div
          class="mt-5 rounded-md border-l-2 border-success bg-success/5 p-3 text-sm text-muted-foreground"
        >
          {t(appsMessages.notProcesses)}
        </div>
      </CardContent>
    </Card>

    <AppSecretsCard {app} />

    <ConfirmOperationAlertDialog
      open={confirmDelete}
      onOpenChange={open => (confirmDelete = open)}
      onContinue={handleDelete}
      title={t(appsMessages.deleteAppTitle)}
      description={t(appsMessages.deleteAppBody)}
    />
  </div>
{/if}

{#snippet appIdLabel()}{t(appsMessages.appId)}{/snippet}
{#snippet nameLabel()}{t(appsMessages.name)}{/snippet}
{#snippet statusLabel()}{t(appsMessages.status)}{/snippet}
{#snippet createdLabel()}{t(appsMessages.createdOn)}{/snippet}
{#snippet updatedLabel()}{t(appsMessages.updatedOn)}{/snippet}
