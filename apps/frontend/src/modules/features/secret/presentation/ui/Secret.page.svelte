<script lang="ts">
  import { createQuery } from '@platform/query';
  import { secretQueries } from '../secret.queries.ts';
  import CreateSecretDialog from './CreateSecretDialog/CreateSecretDialog.svelte';
  import SecretHeader from './components/SecretHeader.svelte';
  import SecretList from './components/SecretList/SecretList.svelte';

  interface Props {
    /** From the route: `/:organizationSlug/projects/:projectId/secrets`. */
    projectId?: string;
  }

  let { projectId }: Props = $props();

  // `enabled` on the query handles a missing id, so the component tree below
  // does not have to — but the id is non-optional to its children, so the page
  // still guards before rendering them.
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
