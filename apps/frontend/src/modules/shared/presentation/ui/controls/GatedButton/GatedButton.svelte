<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { Button, Tooltip, TooltipContent, TooltipTrigger, type ButtonSize, type ButtonVariant } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { gatedButtonMessages } from '../gated-button.messages.ts';

  type Props = HTMLButtonAttributes & {
    /** False disables the button and shows `deniedReason` on hover. */
    allowed?: boolean;
    deniedReason?: string;
    /** Hover text while the button is usable, for an icon-only button. */
    tooltip?: string;
    variant?: ButtonVariant;
    size?: ButtonSize;
    children?: Snippet;
  };

  let {
    allowed = true,
    deniedReason,
    tooltip,
    class: className,
    children,
    ...rest
  }: Props = $props();
</script>

<!--
  A disabled button that says why. The span around the button carries the tooltip:
  a disabled button fires no pointer events.
-->
{#if allowed && !tooltip}
  <Button class={className} {...rest}>{@render children?.()}</Button>
{:else}
  <Tooltip>
    <TooltipTrigger>
      {#snippet child({ props })}
        <span {...props} class="inline-flex">
          <Button
            {...rest}
            disabled={!allowed || rest.disabled}
            class={cn(className, !allowed && 'pointer-events-none')}
          >
            {@render children?.()}
          </Button>
        </span>
      {/snippet}
    </TooltipTrigger>
    <TooltipContent>
      <p>{allowed ? tooltip : (deniedReason ?? t(gatedButtonMessages.notPermitted))}</p>
    </TooltipContent>
  </Tooltip>
{/if}
