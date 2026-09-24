<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';
  import { buttonVariants, type ButtonSize, type ButtonVariant } from './button-variants.ts';

  type Props = HTMLButtonAttributes &
    Pick<HTMLAnchorAttributes, 'href'> & {
      variant?: ButtonVariant;
      size?: ButtonSize;
      children?: Snippet;
      /** The rendered element, for the rare caller that has to focus it. */
      ref?: HTMLElement | null;
    };

  let {
    variant = 'default',
    size = 'default',
    class: className,
    href,
    children,
    ref = $bindable(null),
    ...rest
  }: Props = $props();
</script>

<!--
  `href` replaces React's `asChild` + Radix `Slot`, which has no Svelte
  equivalent. It covers the only use the codebase makes of it — a button that is
  really a link — and renders a real anchor, which is what a screen reader and a
  middle-click both need.
-->
{#if href}
  <a
    bind:this={ref}
    {href}
    data-slot="button"
    data-variant={variant}
    data-size={size}
    class={cn(buttonVariants({ variant, size, className }))}
    {...rest as HTMLAnchorAttributes}
  >
    {@render children?.()}
  </a>
{:else}
  <button
    bind:this={ref}
    data-slot="button"
    data-variant={variant}
    data-size={size}
    class={cn(buttonVariants({ variant, size, className }))}
    {...rest}
  >
    {@render children?.()}
  </button>
{/if}
