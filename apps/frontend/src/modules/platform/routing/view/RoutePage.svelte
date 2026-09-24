<script lang="ts">
  import { PageTransition } from '@shared/presentation/ui';
  import { location } from '../runtime/location.svelte.ts';
  import RouteEntry from './RouteEntry.svelte';

  let { animate = false }: { animate?: boolean } = $props();

  const pathname = $derived(location.pathname);
</script>

{#if animate}
  <PageTransition key={pathname}>
    <RouteEntry />
  </PageTransition>
{:else}
  {#key pathname}
    <RouteEntry />
  {/key}
{/if}
