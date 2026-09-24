<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    name: string;
    subtitle?: Snippet;
  }

  let { name, subtitle }: Props = $props();
</script>

<!-- The avatar is the initial: the member lists carry only a username. -->
<span class="flex min-w-0 items-center gap-3">
  <span
    aria-hidden="true"
    class="flex size-9 shrink-0 items-center justify-center rounded-full bg-primary/10 text-sm font-semibold uppercase text-primary"
  >
    {name.charAt(0)}
  </span>
  <span class="flex min-w-0 flex-col gap-0.5">
    <span class="truncate text-sm font-semibold leading-tight" title={name}>{name}</span>
    {#if subtitle}
      <span class="truncate text-xs leading-tight text-muted-foreground">{@render subtitle()}</span>
    {/if}
  </span>
</span>
