<script lang="ts">
  import { Avatar as AvatarPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: AvatarPrimitive.FallbackProps = $props();
</script>

<AvatarPrimitive.Fallback
  data-slot="avatar-fallback"
  class={cn('flex h-full w-full items-center justify-center rounded-full bg-muted', className)}
  {...rest}
/>
