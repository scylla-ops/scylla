<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';

  type Props = HTMLInputAttributes & { value?: string | number | null };

  let { class: className, value = $bindable(), ...rest }: Props = $props();
</script>

<!-- `bind:value` replaces React's controlled `value` + `onChange` pair. -->
<input
  data-slot="input"
  bind:value
  class={cn(
    'file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm',
    'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]',
    'aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive',
    className,
  )}
  {...rest}
/>
