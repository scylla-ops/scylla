<script lang="ts">
  import { createMutation } from '@platform/query';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { pipelineMutations } from '../../pipeline.queries.ts';
  import { createDefaultScript } from '../../utils/create-default-script.ts';
  import PipelineEditor from './PipelineEditor.svelte';
  import { pipelineMessages } from '../../pipeline.messages.ts';

  interface Props {
    projectId?: string;
  }

  let { projectId }: Props = $props();

  const createPipeline = createMutation(() => pipelineMutations.create());

  const initialScript = $derived(projectId ? createDefaultScript(projectId) : undefined);
</script>

{#if !projectId}
  <p>{t(pipelineMessages.selectProjectFirst)}</p>
{:else}
  <div class="flex h-full flex-col gap-4">
    <PipelineEditor
      mode="create"
      submitLabel={t(pipelineMessages.create)}
      {projectId}
      {initialScript}
      onSubmit={({ name, steps }) => createPipeline.mutate({ name, projectId, nodes: steps })}
      isSubmitPending={createPipeline.isPending}
    />
  </div>
{/if}
