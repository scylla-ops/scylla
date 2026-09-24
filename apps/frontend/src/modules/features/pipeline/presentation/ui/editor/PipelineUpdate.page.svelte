<script lang="ts">
  import { createMutation, createQuery } from '@platform/query';
  import { ErrorState } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { pipelineMutations, pipelineQueries } from '../../pipeline.queries.ts';
  import PipelineEditor from './PipelineEditor.svelte';
  import { pipelineMessages } from '../../pipeline.messages.ts';

  interface Props {
    /** From the route: `edit/:pipelineId`. */
    pipelineId?: string;
  }

  let { pipelineId }: Props = $props();

  const pipelineQuery = createQuery(() => pipelineQueries.byId(pipelineId ?? ''));
  const updatePipeline = createMutation(() => pipelineMutations.update());

  const pipeline = $derived(pipelineQuery.data);

  /**
   * The fetched pipeline, as the editor's document.
   *
   * Only the three fields the editor owns — an id and timestamps would show up
   * in the text area and travel back into `edit()` on the next save.
   */
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
