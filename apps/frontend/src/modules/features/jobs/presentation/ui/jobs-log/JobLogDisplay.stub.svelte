<script lang="ts">
  interface Props {
    jobId: string;
    nodeId?: string;
    maxHeight?: number;
  }

  let { jobId, nodeId, maxHeight }: Props = $props();
</script>

<!-- The viewer reduced to what the details page decides. A component: `vi.mock` of a `.svelte` needs one. -->
<div data-testid="job-log-display" data-max-height={maxHeight}>
  logs for {jobId}/{nodeId ?? 'whole job'}
</div>
