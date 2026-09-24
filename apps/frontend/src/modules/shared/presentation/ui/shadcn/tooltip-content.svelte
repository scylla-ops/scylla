<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Tooltip as TooltipPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  type Props = TooltipPrimitive.ContentProps & { children?: Snippet };

  let { class: className, sideOffset = 0, children, ...rest }: Props = $props();
</script>

<TooltipPrimitive.Portal>
  <!--
    `role="tooltip"` is ours, not bits-ui's: Radix set it and bits-ui does not.
    The trigger's `aria-describedby` is what actually carries the text to a
    screen reader either way, so nothing was broken — but the role is the
    convention, it is what assistive tech uses to classify the node, and it is
    how every tooltip assertion in the suite finds this element.
  -->
  <TooltipPrimitive.Content
    data-slot="tooltip-content"
    role="tooltip"
    {sideOffset}
    class={cn(
      // Copied from `shadcn/tooltip.tsx`, with one substitution: the transform
      // origin comes from bits-ui's own variable, not Radix's. Everything else
      // is Tailwind, which does not care who emitted the markup.
      'bg-foreground text-background animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-fit origin-(--bits-floating-transform-origin) rounded-md px-3 py-1.5 text-xs text-balance',
      className,
    )}
    {...rest}
  >
    {@render children?.()}
    <!--
      The arrow is the wrapper element alone — a rotated, rounded square — which
      is how the React original got its soft point too, and it inherits the
      tooltip background rather than being a second shape to keep in sync.
      `[&>svg]:hidden` drops bits-ui's default triangle, which would otherwise
      draw on top of it.
    -->
    <TooltipPrimitive.Arrow
      class="bg-foreground fill-foreground z-50 size-2.5 translate-y-[calc(-50%_-_2px)] rotate-45 rounded-[2px] [&>svg]:hidden"
    />
  </TooltipPrimitive.Content>
</TooltipPrimitive.Portal>
