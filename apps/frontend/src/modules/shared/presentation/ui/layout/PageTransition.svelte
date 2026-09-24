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
  Replaces `AnimatedOutlet.tsx`, and gets back what the Lot A cleanup lost.

  React could only animate a page *in*: by the time a route changed, the old
  node was already unmounted and no CSS could touch it. Svelte keeps the
  outgoing node alive for the length of its `out:` transition, so both halves
  exist at once — which is also why the wrapper is `relative` and the two panes
  are absolutely stacked: without that, the departing page would occupy layout
  space and shove the arriving one down for 140 ms.

  The two overlap instead of queueing. framer-motion's `mode='wait'` held the
  new page back until the old one had finished leaving, and that 200 ms of
  nothing was the only part anyone noticed.
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
