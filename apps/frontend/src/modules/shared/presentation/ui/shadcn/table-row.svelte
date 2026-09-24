<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLAttributes<HTMLTableRowElement> & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<tr
  data-slot="table-row"
  class={cn(
    'transition-colors duration-150 hover:bg-muted/40 has-aria-expanded:bg-muted/40 data-[state=selected]:bg-primary/[0.06]',
    className,
  )}
  {...rest}
>
  {@render children?.()}
</tr>
