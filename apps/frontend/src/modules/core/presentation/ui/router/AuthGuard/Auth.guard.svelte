<script lang="ts">
  import type { Snippet } from 'svelte';
  import { navigateTo } from '@platform/context';
  import { routePathname } from '@platform/routing';

  let { children }: { children: Snippet } = $props();

  const authenticated = $derived((routePathname(), !!localStorage.getItem('token')));

  $effect(() => {
    if (!authenticated) navigateTo('/login', { replace: true });
  });
</script>

{#if authenticated}
  {@render children()}
{/if}
