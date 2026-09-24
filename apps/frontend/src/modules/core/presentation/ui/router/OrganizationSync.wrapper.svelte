<script lang="ts">
  import { untrack, type Snippet } from 'svelte';
  import type { RouteParams } from '@platform/routing';
  import { syncOrganization } from './organization-sync.svelte.ts';

  let { params, children }: { params: RouteParams; children: Snippet } = $props();

  syncOrganization(untrack(() => params.organizationSlug));
</script>

{@render children()}
