<script lang="ts">
  import KeyRoundIcon from '@lucide/svelte/icons/key-round';
  import PlusIcon from '@lucide/svelte/icons/plus';
  import { can, Permission } from '@platform/authz';
  import { scyllaNavigate, contextStore } from '@platform/context';
  import { createMutation, createQuery } from '@platform/query';
  import { Card, Skeleton } from '@shadcn';
  import {
    ConfirmOperationAlertDialog,
    ErrorState,
    FeatureHeader,
    FormDialog,
    GatedButton,
    SecretRevealDialog,
    type FormValues,
  } from '@shared/presentation/ui';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { activeLocale, t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { CreatedApp } from '../../domain/structs/app.struct.ts';
  import { appMutations, appQueries } from '../apps.queries.ts';
  import { createAppItems } from '../utils/create-app-form-items.ts';
  import AppCard from './components/AppCard.svelte';
  import { appsMessages } from './apps.messages.ts';

  // The organization comes from the context store, not the route: this page is
  // mounted under the org shell and the store is what the selector writes to.
  const context = toRune(contextStore);
  const organizationId = $derived(context().organization.id ?? '');

  const appsQuery = createQuery(() => appQueries.byOrganization(organizationId));
  const createApp = createMutation(() => appMutations.create(organizationId));
  const deleteApp = createMutation(() => appMutations.remove(organizationId));

  const apps = $derived(appsQuery.data ?? []);
  const activeCount = $derived(apps.filter(app => app.isActive).length);

  let createOpen = $state(false);
  /** The one-time plaintext, held only as long as the dialog shows it. */
  let created = $state<CreatedApp | null>(null);
  /** The app awaiting confirmation. `null` closes the dialog. */
  let pendingDeletion = $state<string | null>(null);

  const canCreate = $derived(can(Permission.CREATE_APP));
  const canDelete = $derived(can(Permission.DELETE_APP));

  const items = $derived((activeLocale(), createAppItems()));

  const handleCreate = ({ name }: FormValues<'name'>) => {
    if (!name.trim()) return;

    createApp.mutate(name.trim(), {
      onSuccess: data => {
        createOpen = false;
        created = data;
      },
    });
  };

  const confirmDelete = () => {
    if (pendingDeletion) deleteApp.mutate(pendingDeletion);
    pendingDeletion = null;
  };

  const openCreated = () => {
    const appId = created?.app.id;
    created = null;
    if (appId) scyllaNavigate.goToSubRoute(appId);
  };
</script>

{#if appsQuery.isError}
  <ErrorState message={t(appsMessages.loadError)} />
{:else}
  <div class="flex flex-col w-full h-full overflow-hidden">
    <div class="px-2 pt-2">
      <p class="mb-1 font-mono text-xs uppercase tracking-wider text-muted-foreground">
        {t(appsMessages.service)}
      </p>
      <FeatureHeader
        count={apps.length}
        label={t(appsMessages.app)}
        pluralLabel={t(appsMessages.apps)}
        onNew={() => (createOpen = true)}
        newLabel={t(appsMessages.newApp)}
        canNew={canCreate}
        newDeniedReason={t(appsMessages.createDenied)}
      />
      {#if !appsQuery.isLoading}
        <p class="mt-1 font-mono text-xs text-muted-foreground">
          {activeCount}
          {t(appsMessages.active)} · {apps.length - activeCount}
          {t(appsMessages.inactive)}
        </p>
      {/if}
    </div>

    {#if appsQuery.isLoading}
      <div class="grid gap-3 p-2 sm:grid-cols-2 lg:grid-cols-3">
        {#each { length: 6 } as _, index (index)}
          <Skeleton class="h-32 w-full rounded-xl" />
        {/each}
      </div>
    {:else if apps.length === 0}
      <div class="flex flex-1 items-center justify-center p-6">
        <Card class="flex max-w-sm flex-col items-center gap-3 p-8 text-center">
          <span
            class="flex h-14 w-14 items-center justify-center rounded-xl border border-dashed border-success/40 bg-success/5"
          >
            <KeyRoundIcon class="h-6 w-6 text-success" />
          </span>
          <h2 class="text-lg font-semibold">{t(appsMessages.noAppsYet)}</h2>
          <p class="text-sm text-muted-foreground">{t(appsMessages.noAppsBody)}</p>
          <GatedButton
            allowed={canCreate}
            deniedReason={t(appsMessages.createDenied)}
            onclick={() => (createOpen = true)}
          >
            {t(appsMessages.createFirstApp)}
          </GatedButton>
        </Card>
      </div>
    {:else}
      <div class="grid gap-3 p-2 overflow-y-auto sm:grid-cols-2 lg:grid-cols-3">
        {#each apps as app (app.id)}
          <AppCard {app} onRequestDelete={id => (pendingDeletion = id)} {canDelete} />
        {/each}
        <!-- The header already shows a disabled "New" without CREATE_APP, so
             this duplicate tile is hidden rather than disabled. -->
        {#if canCreate}
          <button
            type="button"
            onclick={() => (createOpen = true)}
            class="flex min-h-32 items-center justify-center gap-2 rounded-xl border border-dashed text-sm text-muted-foreground transition-colors hover:bg-accent/50 hover:text-foreground"
          >
            <PlusIcon class="h-4 w-4" />
            {t(appsMessages.newApp)}
          </button>
        {/if}
      </div>
    {/if}

    <FormDialog
      open={createOpen}
      onOpenChange={open => (createOpen = open)}
      title={t(appsMessages.newApp)}
      description={t(appsMessages.getCredentials)}
      {items}
      isPending={createApp.isPending}
      submitLabel={t(appsMessages.createAndReveal)}
      onSubmit={handleCreate}
    />

    {#if created}
      <SecretRevealDialog
        open
        title={t(appsMessages.credentialsOf(created.app.name))}
        description={t(appsMessages.copySecretOnce)}
        secret={created.secret}
        secretLabel={created.app.id}
        revealedNote={t(appsMessages.useTheseCredentials)}
        onClose={openCreated}
      />
    {/if}

    <ConfirmOperationAlertDialog
      open={pendingDeletion !== null}
      onOpenChange={open => {
        if (!open) pendingDeletion = null;
      }}
      onContinue={confirmDelete}
      title={t(appsMessages.deleteAppTitle)}
      description={t(appsMessages.deleteAppBody)}
    />
  </div>
{/if}
