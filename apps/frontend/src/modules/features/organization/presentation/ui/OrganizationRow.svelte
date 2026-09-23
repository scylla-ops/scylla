<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    class?: string;
    onSelect?: () => void;
    children?: Snippet;
  }

  let { class: className, onSelect, children }: Props = $props();
</script>

<div class={className} role="presentation" onclick={onSelect}>
  {@render children?.()}
</div>
