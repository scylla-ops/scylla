<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLAttributes<HTMLTableSectionElement> & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<thead data-slot="table-header" class={cn(className)} {...rest}>
  {@render children?.()}
</thead>
