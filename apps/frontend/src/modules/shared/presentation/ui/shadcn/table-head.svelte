<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLThAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLThAttributes & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<th
  data-slot="table-head"
  class={cn(
    'h-10 px-4 text-left align-middle text-[0.6875rem] font-semibold tracking-[0.08em] whitespace-nowrap text-muted-foreground uppercase [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</th>
