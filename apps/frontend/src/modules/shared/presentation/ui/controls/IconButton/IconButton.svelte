<script lang="ts">
  import { buttonVariants, Tooltip, TooltipContent, TooltipTrigger } from '@shadcn';
  import { cn } from '@shared/presentation/utils';
  import type { LucideIcon } from '../../icon.ts';

  interface Props {
    icon: LucideIcon;
    tooltip: string;
    onclick?: (event: MouseEvent) => void;
    class?: string;
    iconClass?: string;
    disabled?: boolean;
    /** Disables the button and marks it `aria-busy`. */
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
  The tooltip text is also in the button, visually hidden: a closed tooltip gives
  the button no accessible name.
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
