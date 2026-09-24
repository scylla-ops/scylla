<script lang="ts">
  import { Dialog as DialogPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: DialogPrimitive.TitleProps = $props();
</script>

<DialogPrimitive.Title
  data-slot="dialog-title"
  class={cn('text-lg font-semibold leading-none tracking-tight', className)}
  {...rest}
/>
