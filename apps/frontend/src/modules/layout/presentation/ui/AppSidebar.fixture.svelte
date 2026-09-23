<script lang="ts">
  import type { NavEntry } from '@platform/routing';
  import { SidebarProvider } from '@shadcn-svelte';
  import AppSidebar from './AppSidebar.svelte';

  let { navEntries }: { navEntries: readonly NavEntry[] } = $props();
</script>

<SidebarProvider>
  <AppSidebar {navEntries} />
</SidebarProvider>
