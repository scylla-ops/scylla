<script lang="ts">
  import { loadJobsPage } from '@/modules/features/jobs';
  import { createRunPipeline } from '../run-pipeline.svelte.ts';

  interface Props {
    pipelineId?: string;
  }

  let { pipelineId }: Props = $props();

  const runPipeline = createRunPipeline();
</script>

<!-- `jobs` renders the list; the Run action is composed here, so the dependency stays pipeline → jobs. -->
{#await loadJobsPage() then page}
  <page.default {pipelineId} onRun={() => runPipeline.run(pipelineId ?? '')} />
{/await}
