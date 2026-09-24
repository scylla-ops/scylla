<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '@shared/presentation/utils';

  interface Props {
    children: Snippet;
    class?: string;
  }

  let { children, class: className }: Props = $props();
</script>

<!-- `<nav>` already carries the role; shadcn's React original states it anyway. -->
<nav
  aria-label="pagination"
  data-slot="pagination"
  class={cn('mx-auto flex w-full justify-center', className)}
>
  {@render children()}
</nav>
