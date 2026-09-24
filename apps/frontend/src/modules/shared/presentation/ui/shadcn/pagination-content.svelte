<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '@shared/presentation/utils';

  interface Props {
    children: Snippet;
    class?: string;
  }

  let { children, class: className }: Props = $props();
</script>

<ul data-slot="pagination-content" class={cn('flex flex-row items-center gap-1', className)}>
  {@render children()}
</ul>
