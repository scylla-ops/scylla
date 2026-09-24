<script lang="ts">
  import { cn } from '@shared/presentation/utils';
  import {
    statusIndicatorSize,
    statusStateColors,
    type StatusIndicatorSize,
    type StatusState,
  } from './status-indicator.ts';

  interface Props {
    state?: StatusState;
    label?: string;
    class?: string;
    size?: StatusIndicatorSize;
    labelClass?: string;
    /** Pulses for every state, not only the running ones. */
    animateAllStates?: boolean;
  }

  let {
    state = 'idle',
    label,
    class: className,
    size = 'md',
    labelClass,
    animateAllStates = false,
  }: Props = $props();

  const colors = $derived(statusStateColors(state));
  const sizeClasses = $derived(statusIndicatorSize(size));
  const shouldAnimate = $derived(state === 'running' || state === 'pending' || animateAllStates);
</script>

<!--
  A colored dot, pulsing while the thing is still running. `data-state` lets a
  test read the status without a CSS class.
-->
<div class="relative inline-flex overflow-hidden rounded-full">
  <div
    data-slot="status-indicator"
    data-state={state}
    class={cn(
      'relative inline-flex items-center gap-2 rounded-full bg-card transition-all duration-300',
      sizeClasses.container,
      colors.container,
      className,
    )}
  >
    <div class="relative flex items-center">
      {#if shouldAnimate}
        <span
          class={cn(
            'absolute inline-flex animate-ping rounded-full opacity-75',
            sizeClasses.dot,
            colors.ping,
          )}
        ></span>
      {/if}
      <span class={cn('relative inline-flex rounded-full', sizeClasses.dot, colors.dot)}></span>
    </div>

    {#if label}
      <p class={cn('font-medium', labelClass)}>{label}</p>
    {/if}
  </div>
</div>
