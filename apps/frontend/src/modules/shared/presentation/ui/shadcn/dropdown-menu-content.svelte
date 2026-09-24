<script lang="ts">
  import type { Snippet } from 'svelte';
  import { DropdownMenu as DropdownMenuPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  type Props = DropdownMenuPrimitive.ContentProps & { children?: Snippet };

  let { class: className, sideOffset = 4, children, ...rest }: Props = $props();
</script>

<DropdownMenuPrimitive.Portal>
  <DropdownMenuPrimitive.Content
    data-slot="dropdown-menu-content"
    {sideOffset}
    class={cn(
      // `--radix-dropdown-menu-content-*` become bits-ui's own names. Miss one
      // and the panel animates from the wrong corner, silently.
      'bg-popover text-popover-foreground data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 max-h-(--bits-dropdown-menu-content-available-height) min-w-[8rem] origin-(--bits-floating-transform-origin) overflow-x-hidden overflow-y-auto rounded-md border p-1 shadow-md',
      className,
    )}
    {...rest}
  >
    {@render children?.()}
  </DropdownMenuPrimitive.Content>
</DropdownMenuPrimitive.Portal>
