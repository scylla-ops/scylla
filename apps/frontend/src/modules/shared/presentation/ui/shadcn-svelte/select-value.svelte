<script lang="ts">
  import { Select as SelectPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: SelectPrimitive.ValueProps = $props();
</script>

<!--
  bits-ui's `Value` already renders the selected item's label, or the
  placeholder when there is none — the part `shadcn/select.tsx` had to get from
  Radix. Only the slot attribute is ours, and the trigger's classes select on it.
-->
<SelectPrimitive.Value data-slot="select-value" class={cn(className)} {...rest} />
