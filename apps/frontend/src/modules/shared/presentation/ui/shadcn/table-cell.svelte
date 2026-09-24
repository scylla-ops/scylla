<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLTdAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLTdAttributes & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<td
  data-slot="table-cell"
  class={cn(
    'border-b border-border/50 px-4 py-3 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</td>
