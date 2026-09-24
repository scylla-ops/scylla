<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '@shared/presentation/utils';
  import { pageIn, pageOut } from '../motion/page-transition.ts';

  interface Props {
    /**
     * Changing this replays the transition. The router's pathname, in practice
     * — passed in rather than read here, because the Svelte router does not
     * exist until Phase 6 and this must be usable, and testable, before then.
     */
    key: string;
    children: Snippet;
    class?: string;
  }

  let { key, children, class: className }: Props = $props();
</script>

<!--
  Svelte keeps the outgoing node alive for its `out:` transition, so both panes
  exist at once. They are absolutely stacked, or the departing page would push
  the arriving one down. The arriving page stays transparent until the departing
  one is gone (see `page-transition.ts`), so the text of the two never overlaps.
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
