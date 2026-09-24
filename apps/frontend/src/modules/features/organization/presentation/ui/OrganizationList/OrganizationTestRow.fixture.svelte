<script lang="ts">
  import type { Snippet } from 'svelte';

  let { onSelect, children }: { onSelect?: () => void; children?: Snippet } =
    $props();
</script>

<div data-testid="custom-row">
  <button type="button" onclick={onSelect}>pick</button>
  {@render children?.()}
</div>
