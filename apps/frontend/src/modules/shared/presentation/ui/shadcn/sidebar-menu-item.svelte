<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLLiAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLLiAttributes & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<li
  data-slot="sidebar-menu-item"
  data-sidebar="menu-item"
  class={cn(
    'group/menu-item relative',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</li>
