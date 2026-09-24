<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';
  import Button from './button.svelte';

  type Props = HTMLButtonAttributes & { children?: Snippet };

  let { class: className, children, ...rest }: Props = $props();
</script>

<!-- Plain button for the same reason as `alert-dialog-action.svelte`. -->
<Button variant="outline" class={cn('mt-2 sm:mt-0', className)} {...rest}>
  {@render children?.()}
</Button>
