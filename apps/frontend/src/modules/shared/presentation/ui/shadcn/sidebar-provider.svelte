<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';
  import { setSidebar } from './sidebar-state.svelte.ts';

  type Props = HTMLAttributes<HTMLDivElement> & { children?: Snippet };

  let { class: className, style, children, ...rest }: Props = $props();

  const sidebar = setSidebar();

  const onkeydown = (event: KeyboardEvent) => {
    if (event.key === 'b' && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      sidebar.toggle();
    }
  };
</script>

<svelte:window {onkeydown} />

<div
  data-slot="sidebar-wrapper"
  style="--sidebar-width: 16rem; --sidebar-width-icon: 3rem; {style ?? ''}"
  class={cn(
    'group/sidebar-wrapper has-data-[variant=inset]:bg-sidebar flex min-h-svh w-full',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</div>
