<script lang="ts">
  import { onMount } from 'svelte';
  import { navigateTo } from '@platform/context';

  let { to }: { to: string } = $props();

  onMount(() => navigateTo(to, { replace: true }));
</script>
