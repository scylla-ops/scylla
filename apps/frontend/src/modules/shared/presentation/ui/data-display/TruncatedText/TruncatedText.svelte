<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { cn } from '@shared/presentation/utils';

  interface Props {
    children: Snippet;
    /** Defaults to `children`. */
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
  Shows a tooltip only when the text is really cut off, measured on hover.
  Needs a bounded parent (`min-w-0`).
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
