<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { RouteParams } from '../../../declaration/scylla-module.struct.ts';

  let { params, children }: { params: RouteParams; children: Snippet } = $props();
</script>

<section data-testid="wrapper" data-slug={params.slug}>{@render children()}</section>
