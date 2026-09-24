<script lang="ts">
  import { createQuery } from '@platform/query';
  import { secretQueries } from '../secret.queries.ts';
  import CreateSecretDialog from './CreateSecretDialog/CreateSecretDialog.svelte';
  import SecretHeader from './components/SecretHeader.svelte';
  import SecretList from './components/SecretList/SecretList.svelte';

  interface Props {
    projectId?: string;
  }

  let { projectId }: Props = $props();

  // The children need a defined id, so the page still checks it.
  const secretsQuery = createQuery(() => secretQueries.byProject(projectId ?? ''));
  const secrets = $derived(secretsQuery.data ?? []);

  let isCreateDialogOpen = $state(false);
</script>

{#if projectId}
  <div class="flex min-h-full w-full flex-col gap-4">
    <SecretHeader
      {projectId}
      activeCount={secrets.length}
      secretIds={secrets.map(secret => secret.id)}
      onAddSecret={() => (isCreateDialogOpen = true)}
    />
    <div class="overflow-hidden">
      <SecretList {secrets} {projectId} />
      <CreateSecretDialog
        {projectId}
        isOpen={isCreateDialogOpen}
        setOpen={open => (isCreateDialogOpen = open)}
      />
    </div>
  </div>
{/if}
