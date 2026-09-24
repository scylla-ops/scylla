<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { cn } from '@shared/presentation/utils';

  interface Props {
    children: Snippet;
    /** Tooltip body; defaults to `children`. */
    tooltip?: string;
    class?: string;
  }

  let { children, tooltip, class: className }: Props = $props();

  let isTruncated = $state(false);

  const measure = (event: Event) => {
    const element = event.currentTarget as HTMLElement;
    isTruncated = element.scrollWidth > element.clientWidth;
  };
</script>

<!--
  Single-line text that ellipsizes, and reveals itself in a tooltip *only* when it
  is actually cut off — a tooltip repeating text already fully visible is noise.

  The overflow is measured when the pointer arrives rather than watched with a
  ResizeObserver: a resize while nothing hovers the text changes nothing the user
  can see, and this keeps the component free of subscriptions in every table row.
  That is also why this is an event handler and not a Svelte action — there is no
  outside system to stay in sync with, only a question asked on hover.

  Needs a bounded parent to have any effect — a flex/grid ancestor with `min-w-0`.
-->
<Tooltip>
  <TooltipTrigger
    onpointerenter={measure}
    onfocus={measure}
    class={cn('block min-w-0 truncate text-left', className)}
  >
    {@render children()}
  </TooltipTrigger>
  {#if isTruncated}
    <TooltipContent>
      {#if tooltip}
        {tooltip}
      {:else}
        {@render children()}
      {/if}
    </TooltipContent>
  {/if}
</Tooltip>
