<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { Button, Tooltip, TooltipContent, TooltipTrigger, type ButtonSize, type ButtonVariant } from '@shadcn-svelte';
  import { cn } from '@shared/presentation/utils';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { gatedButtonMessages } from './gated-button.messages.ts';

  type Props = HTMLButtonAttributes & {
    /** False shows the button disabled, with {@link deniedReason} on hover. */
    allowed?: boolean;
    deniedReason?: string;
    /** Hover text while the button *is* usable — for an icon-only control. */
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
  A button that explains itself when the user may not use it — the middle ground
  between hiding an action and letting someone click something that can only
  fail server-side.

  It takes a plain boolean rather than a `Permission`, which is what lets it
  live in `shared/`: the caller asks `can(…)` and passes the answer, and the
  component keeps no business meaning. `PermissionButton` in `@platform/authz`
  is its React twin plus that lookup.

  The span between trigger and button is not decoration: a disabled button fires
  no pointer events, so a tooltip triggered by the button itself would never
  open — which is precisely when its explanation is needed.
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
