<script lang="ts">
  import { DropdownMenu as DropdownMenuPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: DropdownMenuPrimitive.SeparatorProps = $props();
</script>

<DropdownMenuPrimitive.Separator
  data-slot="dropdown-menu-separator"
  class={cn('bg-border -mx-1 my-1 h-px', className)}
  {...rest}
/>
