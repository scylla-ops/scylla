<script lang="ts">
  import type { Component } from 'svelte';
  import { SidebarProvider } from '@shadcn';

  let { component: Inner }: { component: Component } = $props();
</script>

<SidebarProvider>
  <Inner />
</SidebarProvider>
