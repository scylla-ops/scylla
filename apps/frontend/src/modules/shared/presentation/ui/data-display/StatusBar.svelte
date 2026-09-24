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
    /** The tooltip of every segment, with its item. Without it, no tooltips. */
    tooltip?: Snippet<[TItem]>;
    emptyLabel?: string;
    class?: string;
    /** A Tailwind height class. */
    height?: string;
  }

  let { items, tooltip, emptyLabel, class: className, height = 'h-6' }: Props = $props();

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

<!-- A bar of colored segments, one per status (job history, node timelines). -->
{#snippet segment(item: TItem, trigger: TriggerProps = {})}
  {#if item.onSelect}
    <button
      type="button"
      aria-label={item.label}
      {...trigger}
      onclick={event => {
        // Call the trigger's handler too: it closes the tooltip on click.
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
