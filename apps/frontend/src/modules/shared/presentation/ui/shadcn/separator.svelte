<script lang="ts">
  import { Separator as SeparatorPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let {
    class: className,
    orientation = 'horizontal',
    ...rest
  }: SeparatorPrimitive.RootProps = $props();
</script>

<SeparatorPrimitive.Root
  data-slot="separator"
  {orientation}
  class={cn(
    'bg-border shrink-0 data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px',
    className,
  )}
  {...rest}
/>
