<script lang="ts">
  import { RadioGroup as RadioGroupPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let {
    class: className,
    value = $bindable(''),
    ...rest
  }: RadioGroupPrimitive.RootProps = $props();
</script>

<RadioGroupPrimitive.Root
  data-slot="radio-group"
  bind:value
  class={cn('grid gap-3', className)}
  {...rest}
/>
