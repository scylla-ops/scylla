<script lang="ts">
  import { AlertDialog as AlertDialogPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: AlertDialogPrimitive.TitleProps = $props();
</script>

<AlertDialogPrimitive.Title
  data-slot="alert-dialog-title"
  class={cn('text-lg font-semibold', className)}
  {...rest}
/>
