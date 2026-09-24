<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLAttributes<HTMLUListElement> & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<ul
  data-slot="sidebar-menu"
  data-sidebar="menu"
  class={cn(
    'flex w-full min-w-0 flex-col gap-1',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</ul>
