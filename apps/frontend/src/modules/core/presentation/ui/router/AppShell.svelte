<script lang="ts">
  import type { Snippet } from 'svelte';
  import { navEntriesFor } from '@platform/routing';
  import Layout from '@/modules/layout/presentation/ui/Layout.svelte';
  import { modules } from '@core/di/registry.ts';
  import AuthGuard from './Auth.guard.svelte';

  let { children }: { children: Snippet } = $props();

  const navEntries = navEntriesFor(modules);
</script>

<AuthGuard>
  <Layout {navEntries}>
    {@render children()}
  </Layout>
</AuthGuard>
