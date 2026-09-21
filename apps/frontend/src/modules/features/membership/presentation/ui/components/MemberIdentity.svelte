<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    name: string;
    /** The line under the name — on a card, how many roles they hold. */
    subtitle?: Snippet;
  }

  let { name, subtitle }: Props = $props();
</script>

<!--
  A member's name, with the initial-avatar that makes a long list scannable.

  The avatar is derived rather than fetched: the backend's member lists carry a
  username and nothing else, and a row of identical placeholder pictures would
  be noise. One letter per person is enough to give the eye an anchor.
-->
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
