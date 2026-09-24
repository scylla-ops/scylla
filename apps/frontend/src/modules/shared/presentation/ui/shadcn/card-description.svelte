<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLAttributes<HTMLDivElement> & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<div data-slot="card-description" class={cn('text-sm text-muted-foreground', className)} {...rest}>
  {@render children?.()}
</div>
