<script lang="ts" generics="TItem extends StatusBarItem">
  import type { Snippet } from 'svelte';
  import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { getStatusConfig } from '@shared/utils/status-config.ts';
  import type { StatusBarItem } from './status-bar.ts';
  import { statusBarMessages } from './status-bar.messages.ts';

  interface Props {
    items: TItem[];
    /**
     * Rendered in every segment's tooltip, with the item it belongs to. One
     * snippet for the whole bar rather than one per item: a snippet cannot be
     * partially applied, so the only way to give it per-segment data is to hand
     * it the segment. Omit it and the bar has no tooltips at all.
     */
    tooltip?: Snippet<[TItem]>;
    emptyLabel?: string;
    class?: string;
    /** A Tailwind height class — the caller decides how tall the bar reads. */
    height?: string;
  }

  let { items, tooltip, emptyLabel, class: className, height = 'h-6' }: Props = $props();

  /** Anything bits-ui's trigger hands down, when the segment is inside one. */
  type TriggerProps = Record<string, unknown> & { onclick?: (event: MouseEvent) => void };

  const segmentClass = (item: TItem) => {
    const config = getStatusConfig(item.status);
    return cn(
      'flex-1 min-w-[2px] max-w-full h-full rounded-sm transition-all duration-150 shrink',
      config.barClassName,
      config.barHoverClassName,
      (tooltip || item.onSelect) && 'cursor-pointer',
    );
  };
</script>

<!--
  A bar of coloured segments, one per status: pipeline job history and job node
  timelines. Ported class for class from `ui/data-display/StatusBar.tsx`.

  Two things changed shape. React could hold a whole element in a variable, so
  the segment was built once and then either returned bare or wrapped in a
  tooltip; here that variable is a snippet, rendered in both arms. And `asChild`
  became the `child` snippet, which is why `trigger` is spread onto *our*
  element — the tooltip must describe the bar itself, not a wrapper around it,
  or the hover target and the coloured segment stop being the same node.
-->
{#snippet segment(item: TItem, trigger: TriggerProps = {})}
  {#if item.onSelect}
    <button
      type="button"
      aria-label={item.label}
      {...trigger}
      onclick={event => {
        // Chained, not replaced: the trigger's own handler is what closes the
        // tooltip on click, and spreading `trigger` before this would otherwise
        // leave it on screen over the page the click navigates to.
        trigger.onclick?.(event);
        event.stopPropagation();
        item.onSelect?.();
      }}
      class={segmentClass(item)}
    ></button>
  {:else}
    <div {...trigger} class={segmentClass(item)}></div>
  {/if}
{/snippet}

{#if items.length === 0}
  <div class={cn('w-full flex items-center justify-center py-1', height)}>
    <span class="text-xs text-muted-foreground italic">
      {emptyLabel ?? t(statusBarMessages.empty)}
    </span>
  </div>
{:else}
  <div
    class={cn(
      'w-full flex items-center gap-[1px] py-1 overflow-hidden rounded-md',
      height,
      className,
    )}
  >
    {#each items as item (item.id)}
      {#if tooltip}
        <Tooltip delayDuration={100}>
          <TooltipTrigger>
            {#snippet child({ props })}
              {@render segment(item, props)}
            {/snippet}
          </TooltipTrigger>
          <TooltipContent
            side="top"
            class="text-xs text-popover-foreground border-border p-3 shadow-lg bg-popover"
          >
            {@render tooltip(item)}
          </TooltipContent>
        </Tooltip>
      {:else}
        {@render segment(item)}
      {/if}
    {/each}
  </div>
{/if}
