<script lang="ts">
  import type { Snippet } from 'svelte';
  import { buttonVariants, type ButtonSize } from './button-variants.ts';
  import { cn } from '@shared/presentation/utils';

  interface Props {
    children: Snippet;
    isActive?: boolean;
    size?: ButtonSize;
    class?: string;
    'aria-label'?: string;
    onclick?: (event: MouseEvent) => void;
  }

  let {
    children,
    isActive = false,
    size = 'icon',
    class: className,
    'aria-label': ariaLabel,
    onclick,
  }: Props = $props();
</script>

<!--
  A `<button>`, where shadcn's React original renders an `<a>` without an href.
  Nothing here navigates — the page is state, not a URL — and an anchor with no
  href is not a control to the accessibility tree, so it could not be found by
  role or reached by keyboard. The classes are unchanged, so it looks identical.
-->
<button
  type="button"
  aria-current={isActive ? 'page' : undefined}
  aria-label={ariaLabel}
  data-slot="pagination-link"
  data-active={isActive}
  class={cn(buttonVariants({ variant: isActive ? 'outline' : 'ghost', size }), className)}
  {onclick}
>
  {@render children()}
</button>
