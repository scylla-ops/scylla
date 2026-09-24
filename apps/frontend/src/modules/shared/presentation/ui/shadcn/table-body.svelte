<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLAttributes<HTMLTableSectionElement> & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<tbody data-slot="table-body" class={cn('[&_tr:last-child>td]:border-b-0', className)} {...rest}>
  {@render children?.()}
</tbody>
