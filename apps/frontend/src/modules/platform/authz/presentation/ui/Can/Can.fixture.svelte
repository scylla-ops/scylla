<script lang="ts">
  import type { Permission } from '../../../domain/structs/permission.struct.ts';
  import Can from './Can.svelte';

  let { permission, withFallback = false }: { permission: Permission; withFallback?: boolean } =
    $props();
</script>

{#if withFallback}
  <Can {permission}>
    <span>visible content</span>
    {#snippet fallback()}
      <span>fallback content</span>
    {/snippet}
  </Can>
{:else}
  <Can {permission}>
    <span>visible content</span>
  </Can>
{/if}
