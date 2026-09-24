<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Select as SelectPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  type Props = SelectPrimitive.ContentProps & { children?: Snippet };

  let { class: className, align = 'center', children, ...rest }: Props = $props();
</script>

<SelectPrimitive.Portal>
  <SelectPrimitive.Content
    data-slot="select-content"
    {align}
    class={cn(
      // The three `--radix-select-*` variables become bits-ui's own. Missing one
      // costs an animation origin or a max height, and nothing fails loudly.
      'bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 relative z-50 max-h-(--bits-select-content-available-height) min-w-[8rem] origin-(--bits-floating-transform-origin) overflow-x-hidden overflow-y-auto rounded-md border shadow-md',
      'data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1',
      className,
    )}
    {...rest}
  >
    <SelectPrimitive.Viewport
      class="h-[var(--bits-select-anchor-height)] w-full min-w-[var(--bits-select-anchor-width)] scroll-my-1 p-1"
    >
      {@render children?.()}
    </SelectPrimitive.Viewport>
  </SelectPrimitive.Content>
</SelectPrimitive.Portal>
