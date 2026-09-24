<script lang="ts">
  import { Avatar as AvatarPrimitive } from 'bits-ui';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: AvatarPrimitive.RootProps = $props();
</script>

<AvatarPrimitive.Root
  data-slot="avatar"
  class={cn('relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full', className)}
  {...rest}
/>
