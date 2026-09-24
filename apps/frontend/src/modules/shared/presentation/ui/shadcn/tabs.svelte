<script lang="ts">
  import { Tabs as TabsPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, value = $bindable(), ...rest }: TabsPrimitive.RootProps = $props();
</script>

<TabsPrimitive.Root
  data-slot="tabs"
  bind:value
  class={cn('flex flex-col gap-2', className)}
  {...rest}
/>
