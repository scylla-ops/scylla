<script lang="ts">
  let props: Record<string, string | undefined> = $props();
</script>

<p data-testid="page">{Object.entries(props).map(([key, value]) => `${key}=${value}`).join(' ')}</p>
