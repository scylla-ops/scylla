<script lang="ts">
  import { Tabs as TabsPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: TabsPrimitive.ContentProps = $props();
</script>

<TabsPrimitive.Content
  data-slot="tabs-content"
  class={cn('flex-1 outline-none data-[state=inactive]:hidden', className)}
  {...rest}
/>
