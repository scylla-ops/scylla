<script lang="ts">
  import { onMount } from 'svelte';
  import { fade } from 'svelte/transition';
  import iconScylla from '@/assets/icon_scylla.png';
  import { motionDuration } from '../motion/reduced-motion.ts';

  /**
   * How long a load may take before it is worth telling the user about. Under
   * this, painting the logo and removing it in the same breath reads as a
   * glitch rather than as progress — so the screen stays empty and a fast load
   * shows nothing at all.
   */
  const LOGO_DELAY_MS = 300;

  let showLogo = $state(false);

  // A timer is a system outside the component — nothing in the state says
  // "300 ms have passed" — so `onMount` is the right tool here, not a
  // derivation. Callers render this while their query is pending, so a fast
  // load unmounts it first and the teardown cancels the timer.
  onMount(() => {
    const timer = setTimeout(() => (showLogo = true), LOGO_DELAY_MS);
    return () => clearTimeout(timer);
  });
</script>

<div class="flex h-screen w-screen items-center justify-center bg-background">
  {#if showLogo}
    <!--
      Two nodes because both animations drive `animation` and would overwrite
      each other on one: the wrapper fades — softening the boundary, so a load
      that ends just after the delay never reaches full opacity — while the logo
      spins. The spin stays CSS, as it did; only the fade became a transition,
      which is what buys the fade *out* React could not do.
    -->
    <span in:fade={{ duration: motionDuration(200) }} out:fade={{ duration: motionDuration(150) }}>
      <img
        src={iconScylla}
        alt="Scylla"
        class="h-28 w-28 animate-[scylla-spin_1.8s_ease-in-out_infinite]"
      />
    </span>
  {/if}
</div>
