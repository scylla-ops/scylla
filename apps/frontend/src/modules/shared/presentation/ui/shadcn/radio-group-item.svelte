<script lang="ts">
  import { RadioGroup as RadioGroupPrimitive } from 'bits-ui';
  import CircleIcon from '@lucide/svelte/icons/circle';
  import { cn } from '@shared/presentation/utils';

  let { class: className, ...rest }: RadioGroupPrimitive.ItemProps = $props();
</script>

<RadioGroupPrimitive.Item
  data-slot="radio-group-item"
  class={cn(
    'aspect-square size-4 shrink-0 rounded-full border border-input text-primary shadow-xs transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:bg-input/30 dark:aria-invalid:ring-destructive/40',
    className,
  )}
  {...rest}
>
  <!--
    Radix's `Indicator` mounted only while checked; bits-ui hands the state to
    the children snippet, so the `{#if}` is what replaces it — same as the
    checkbox and the select item.
  -->
  {#snippet children({ checked })}
    <span
      data-slot="radio-group-indicator"
      class="relative flex size-full items-center justify-center"
    >
      {#if checked}
        <CircleIcon
          class="absolute top-1/2 left-1/2 size-2 -translate-x-1/2 -translate-y-1/2 fill-primary"
        />
      {/if}
    </span>
  {/snippet}
</RadioGroupPrimitive.Item>
