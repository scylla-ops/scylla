<script lang="ts">
  import { PageTransition } from '@shared/presentation/ui-svelte';
  import { routeInShell, routePathname } from './route-state.ts';
  import RouteEntry from './RouteEntry.svelte';

  const pathname = $derived(routePathname());
  const inShell = $derived(routeInShell());
</script>

{#if inShell}
  <PageTransition key={pathname}>
    <RouteEntry />
  </PageTransition>
{:else}
  {#key pathname}
    <RouteEntry />
  {/key}
{/if}
