<script lang="ts">
  import KeyIcon from '@lucide/svelte/icons/key';
  import UsersIcon from '@lucide/svelte/icons/users';
  import { Permission, can } from '@platform/authz';
  import { scyllaNavigate } from '@platform/context';
  import { createMutation } from '@platform/query';
  import { Button } from '@shadcn-svelte';
  import { FeatureHeader } from '@shared/presentation/ui-svelte';
  import { createFeatureSelection } from '@shared/presentation/state/feature-selection.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { pipelineMutations } from '../../pipeline.queries.ts';
  import { pipelineMessages } from '../../pipeline.messages.ts';

  interface Props {
    numberOfPipelines: number;
    pipelineIds: string[];
  }

  let { numberOfPipelines, pipelineIds }: Props = $props();

  const deletePipeline = createMutation(() => pipelineMutations.remove());

  // A getter, not the array: the ids are server data that arrives late, and
  // capturing them once would freeze "select all" on the first, empty render.
  const selection = createFeatureSelection('pipelines', () => pipelineIds, {
    deleteItem: id => deletePipeline.mutateAsync(id),
  });

  const canCreate = $derived(can(Permission.CREATE_PIPELINE));
  const canDelete = $derived(can(Permission.DELETE_PIPELINE));
  const canListMembers = $derived(can(Permission.LIST_PROJECT_MEMBERS));
  const canListSecrets = $derived(can(Permission.LIST_SECRETS));
</script>

<!--
  The project dashboard is where someone stands when they think about who works
  on the project, so its member page opens from here rather than from a separate
  administration screen.
-->
{#snippet projectShortcuts()}
  {#if canListMembers}
    <Button variant="outline" onclick={() => scyllaNavigate.goToSubRoute('members')}>
      <UsersIcon class="text-primary" />
      {t(pipelineMessages.members)}
    </Button>
  {/if}
  {#if canListSecrets}
    <Button variant="outline" onclick={() => scyllaNavigate.goToSubRoute('secrets')}>
      <KeyIcon class="text-primary" />
      {t(pipelineMessages.secrets)}
    </Button>
  {/if}
{/snippet}

<div class="flex w-full items-center gap-4">
  <FeatureHeader
    count={numberOfPipelines}
    label={t(pipelineMessages.pipeline)}
    pluralLabel={t(pipelineMessages.pipelines)}
    onNew={() => scyllaNavigate.goToCreatePipeline()}
    newLabel={t(pipelineMessages.newPipeline)}
    canNew={canCreate}
    newDeniedReason={t(pipelineMessages.createDenied)}
    {canDelete}
    deleteDeniedReason={t(pipelineMessages.deleteDenied)}
    extraActions={projectShortcuts}
    {...selection.headerProps}
  />
</div>
