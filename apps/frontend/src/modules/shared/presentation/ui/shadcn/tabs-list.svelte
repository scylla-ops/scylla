<script lang="ts">
  import { Tabs as TabsPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: TabsPrimitive.ListProps = $props();
</script>

<TabsPrimitive.List
  data-slot="tabs-list"
  class={cn(
    'bg-muted text-muted-foreground inline-flex h-9 w-fit items-center justify-center rounded-lg p-[3px]',
    className,
  )}
  {...rest}
/>
