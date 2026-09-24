<script lang="ts">
  import { listenToLocation } from '../../runtime/location.svelte.ts';
  import { currentMatch, routeFallback } from '../../runtime/route-state.svelte.ts';
  import RoutePage from '../RoutePage.svelte';

  $effect(() => listenToLocation());

  const matched = $derived(currentMatch() !== null);
  const Layout = $derived(currentMatch()?.route.layout);
  const Fallback = $derived(routeFallback());
</script>

{#if !matched}
  <Fallback />
{:else if Layout}
  <Layout>
    <RoutePage animate />
  </Layout>
{:else}
  <RoutePage />
{/if}
