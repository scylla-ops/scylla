<script lang="ts">
  import { loadJobsPage } from '@/modules/features/jobs';
  import { createRunPipeline } from '../run-pipeline.svelte.ts';

  interface Props {
    /** From the route: `pipelines/:pipelineId/jobs`. */
    pipelineId?: string;
  }

  let { pipelineId }: Props = $props();

  const runPipeline = createRunPipeline();
</script>

<!--
  The jobs page as mounted under a pipeline.

  `jobs` renders the list; running the pipeline is a pipeline operation, so the
  composition happens here — on the side already allowed to depend on `jobs`.
  That keeps the dependency one-way (pipeline → jobs) instead of the mutual
  import the "Run" button would otherwise need.

  Awaited, not imported: the barrel hands out a loader so a `.svelte` it
  re-exports cannot be dragged into the chunk of whoever imports the barrel for
  something else. Until Phase 5 this was a `LazySvelteIsland`; the loader is the
  same, only the host is no longer React.
-->
{#await loadJobsPage() then page}
  <page.default {pipelineId} onRun={() => runPipeline.run(pipelineId ?? '')} />
{/await}
