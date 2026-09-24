<script lang="ts">
  import { buttonVariants, Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import type { LucideIcon } from '../icon.ts';

  interface Props {
    icon: LucideIcon;
    tooltip: string;
    onclick?: (event: MouseEvent) => void;
    class?: string;
    iconClass?: string;
    disabled?: boolean;
    /**
     * The action is running. Marks the control `aria-busy` and disables it, so
     * "busy" is one prop rather than a `disabled` and a spinning icon that can
     * drift apart.
     */
    busy?: boolean;
  }

  let {
    icon: Icon,
    tooltip,
    onclick,
    class: className,
    iconClass,
    disabled = false,
    busy = false,
  }: Props = $props();
</script>

<!--
  The trigger *is* the button, rather than a trigger wrapping one through the
  `child` snippet: bits-ui's trigger already renders a `<button>` and merges what
  it is given, so dressing it with `buttonVariants` yields the same single
  element the React `asChild` produced — and `mergeProps` chains our `onclick`
  with the tooltip's instead of one silently replacing the other.

  The tooltip text is also rendered visually-hidden inside the button: a tooltip
  is only wired up as `aria-describedby`, and while it is closed that leaves the
  button with no accessible name at all — unusable by a screen reader, and
  unfindable by name in a test.
-->
<Tooltip>
  <TooltipTrigger
    disabled={disabled || busy}
    aria-busy={busy || undefined}
    {onclick}
    class={cn(
      buttonVariants({ variant: 'ghost', size: 'icon' }),
      'h-8 w-8 cursor-pointer rounded-full transition-all duration-200 hover:scale-125 hover:bg-primary-subtle hover:text-primary active:scale-95',
      className,
    )}
  >
    <Icon class={cn('h-4 w-4', iconClass)} />
    <span class="sr-only">{tooltip}</span>
  </TooltipTrigger>
  <TooltipContent>
    <p>{tooltip}</p>
  </TooltipContent>
</Tooltip>
