<script lang="ts">
  import { createMutation, createQuery } from '@platform/query';
  import { ErrorState } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { pipelineMutations, pipelineQueries } from '../../pipeline.queries.ts';
  import PipelineEditor from './PipelineEditor.svelte';
  import { pipelineMessages } from '../../pipeline.messages.ts';

  interface Props {
    pipelineId?: string;
  }

  let { pipelineId }: Props = $props();

  const pipelineQuery = createQuery(() => pipelineQueries.byId(pipelineId ?? ''));
  const updatePipeline = createMutation(() => pipelineMutations.update());

  const pipeline = $derived(pipelineQuery.data);

  /** Only the fields the editor owns: an id or timestamps would travel back on save. */
  const initialScript = $derived(
    pipeline
      ? JSON.stringify(
          { name: pipeline.name, projectId: pipeline.projectId, nodes: pipeline.nodes },
          null,
          2,
        )
      : undefined,
  );
</script>

{#if pipelineQuery.isLoading}
  <p>{t(pipelineMessages.loading)}</p>
{:else if pipelineQuery.isError}
  <ErrorState message={t(pipelineMessages.loadPipelineError)} />
{:else}
  <div class="flex h-full flex-col gap-4">
    <PipelineEditor
      mode="edit"
      submitLabel={t(pipelineMessages.save)}
      projectId={pipeline?.projectId}
      {initialScript}
      onSubmit={({ name, steps }) => {
        if (pipelineId) updatePipeline.mutate({ id: pipelineId, name, nodes: steps });
      }}
      isSubmitPending={updatePipeline.isPending}
    />
  </div>
{/if}
