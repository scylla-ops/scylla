<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLAttributes<HTMLDivElement> & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<div
  data-slot="sidebar-group"
  data-sidebar="group"
  class={cn(
    'relative flex w-full min-w-0 flex-col p-2',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</div>
