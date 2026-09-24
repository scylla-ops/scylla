<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '@shared/presentation/utils';
  import { fieldVariants, type FieldOrientation } from './field-variants.ts';

  type Props = HTMLAttributes<HTMLDivElement> & {
    orientation?: FieldOrientation;
    children?: Snippet;
  };

  let { class: className, orientation = 'vertical', children, ...rest }: Props = $props();
</script>

<div
  role="group"
  data-slot="field"
  data-orientation={orientation}
  class={cn(fieldVariants({ orientation }), className)}
  {...rest}
>
  {@render children?.()}
</div>
