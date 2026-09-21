<script lang="ts">
  interface Props {
    jobId: string;
    nodeId?: string;
    maxHeight?: number;
  }

  let { jobId, nodeId, maxHeight }: Props = $props();
</script>

<!--
  The viewer, reduced to the two facts the details page is responsible for:
  which panel this is, and how tall it was allowed to grow. The real one opens a
  CodeMirror instance and a live gRPC stream, neither of which that page decides
  anything about — `JobLogDisplay.test.ts` covers those.

  A file rather than an inline factory in the test: `vi.mock` of a `.svelte`
  module has to hand back something the Svelte compiler produced, and only a
  real component is.
-->
<div data-testid="job-log-display" data-max-height={maxHeight}>
  logs for {jobId}/{nodeId ?? 'whole job'}
</div>
