<script lang="ts">
  import { untrack, type Snippet } from 'svelte';
  import type { RouteParams } from '@platform/routing';
  import { cleanContext } from './context-cleaner.svelte.ts';

  let { params, children }: { params: RouteParams; children: Snippet } = $props();

  cleanContext(untrack(() => params.projectId));
</script>

{@render children()}
