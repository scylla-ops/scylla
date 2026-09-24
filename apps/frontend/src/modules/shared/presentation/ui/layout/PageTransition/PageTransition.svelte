<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '@shared/presentation/utils';
  import { pageIn, pageOut } from '../../motion/page-transition.ts';

  interface Props {
    /** A change replays the transition. */
    key: string;
    children: Snippet;
    class?: string;
  }

  let { key, children, class: className }: Props = $props();
</script>

<!--
  The departing and the arriving page exist at once: they are stacked, and the
  arriving one stays transparent until the other is gone (see `page-transition.ts`).
-->
<div class="relative h-full w-full">
  {#key key}
    <main
      in:pageIn
      out:pageOut
      class={cn('absolute inset-0 flex h-full w-full flex-col p-2', className)}
    >
      {@render children()}
    </main>
  {/key}
</div>
