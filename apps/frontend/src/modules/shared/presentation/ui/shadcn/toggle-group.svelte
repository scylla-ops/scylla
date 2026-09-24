<script lang="ts">
  import { ToggleGroup as ToggleGroupPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';
  import {
    setToggleGroupStyle,
    type ToggleSize,
    type ToggleVariant,
  } from './toggle-variants.ts';

  type Props = ToggleGroupPrimitive.RootProps & {
    variant?: ToggleVariant;
    size?: ToggleSize;
    spacing?: number;
  };

  let {
    class: className,
    variant = 'default',
    size = 'default',
    spacing = 0,
    children,
    ...rest
  }: Props = $props();

  // A getter, so an item reads the current value rather than whatever the group
  // was configured with on the render that created the context.
  setToggleGroupStyle(() => ({ variant, size, spacing }));
</script>

<ToggleGroupPrimitive.Root
  data-slot="toggle-group"
  data-variant={variant}
  data-size={size}
  data-spacing={spacing}
  style={`--gap: ${spacing}`}
  class={cn(
    'group/toggle-group flex w-fit items-center gap-[--spacing(var(--gap))] rounded-md data-[spacing=default]:data-[variant=outline]:shadow-xs',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</ToggleGroupPrimitive.Root>
