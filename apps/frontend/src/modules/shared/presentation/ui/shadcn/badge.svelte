<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';
  import { badgeVariants, type BadgeVariant } from './badge-variants.ts';

  type Props = HTMLAttributes<HTMLSpanElement> & {
    variant?: BadgeVariant;
    children?: Snippet;
  };

  let { variant = 'default', class: className, children, ...rest }: Props = $props();
</script>

<!--
  No `asChild`: the React original took one so a badge could become a link, and
  nothing in this codebase ever did. The `[a&]:hover:` arms in the variants are
  kept anyway — they are inert on a span and are what the day someone needs it
  will reach for, the same way `Button` grew an `href`.
-->
<span data-slot="badge" class={cn(badgeVariants({ variant }), className)} {...rest}>
  {@render children?.()}
</span>
