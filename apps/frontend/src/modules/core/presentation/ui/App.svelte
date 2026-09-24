<script lang="ts">
  import { Toaster } from '@shadcn';
  import { RouterView } from '@platform/routing';
  import ThemeToggle from './ThemeToggle.svelte';
</script>

<ThemeToggle />
<RouterView />
<Toaster />
