<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLAttributes<HTMLDivElement> & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<div
  data-slot="alert-dialog-header"
  class={cn('flex flex-col space-y-2 text-center sm:text-left', className)}
  {...rest}
>
  {@render children?.()}
</div>
