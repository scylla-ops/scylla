<script lang="ts">
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import iconScylla from '@/assets/icon_scylla.png';
  import { motionDuration } from '../motion/reduced-motion.ts';

  /** A fast load shows nothing: a logo that flashes reads as a glitch. */
  const LOGO_DELAY_MS = 300;

  let showLogo = $state(false);

  onMount(() => {
    const timer = setTimeout(() => (showLogo = true), LOGO_DELAY_MS);
    return () => clearTimeout(timer);
  });
</script>

<div class="flex h-screen w-screen items-center justify-center bg-background">
  {#if showLogo}
    <!-- Two nodes: the fade and the spin both use `animation`. -->
    <span in:fade={{ duration: motionDuration(200) }} out:fade={{ duration: motionDuration(150) }}>
      <img
        src={iconScylla}
        alt="Scylla"
        class="h-28 w-28 animate-[scylla-spin_1.8s_ease-in-out_infinite]"
      />
    </span>
  {/if}
</div>
