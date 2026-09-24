<script lang="ts">
  import { Label as LabelPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: LabelPrimitive.RootProps = $props();
</script>

<LabelPrimitive.Root
  data-slot="field-label"
  class={cn(
    'flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50',
    'group/field-label peer/field-label flex w-fit gap-2 leading-snug group-data-[disabled=true]/field:opacity-50',
    'has-[>[data-slot=field]]:w-full has-[>[data-slot=field]]:flex-col has-[>[data-slot=field]]:rounded-md has-[>[data-slot=field]]:border [&>*]:data-[slot=field]:p-4',
    'has-data-[state=checked]:bg-primary/5 has-data-[state=checked]:border-primary dark:has-data-[state=checked]:bg-primary/10',
    className,
  )}
  {...rest}
/>
