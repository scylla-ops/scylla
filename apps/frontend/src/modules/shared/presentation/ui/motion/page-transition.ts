import { cubicOut, cubicIn } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';
import { motionDuration } from './reduced-motion.ts';

/** The arriving page stays transparent this long, so the two texts never overlap. */
export const PAGE_OUT_MS = 100;

export const PAGE_IN_MS = 200;

const PAGE_CHANGE_MS = PAGE_OUT_MS + PAGE_IN_MS;

/**
 * Waits out `PAGE_OUT_MS` inside the curve, not with a `delay`: a delayed
 * transition would show the page at full opacity first. `_node` is required by
 * the transition signature.
 */
export const pageIn = (_node?: Element): TransitionConfig => ({
  duration: motionDuration(PAGE_CHANGE_MS),
  css: t => {
    const progress = cubicOut(Math.max(0, (t * PAGE_CHANGE_MS - PAGE_OUT_MS) / PAGE_IN_MS));
    return `opacity: ${progress}; transform: scale(${0.99 + 0.01 * progress})`;
  },
});

/** `PageTransition` takes the node out of the flow while it runs. */
export const pageOut = (_node?: Element): TransitionConfig => ({
  duration: motionDuration(PAGE_OUT_MS),
  easing: cubicIn,
  css: (t, u) => `opacity: ${t}; transform: scale(${1 + 0.006 * u})`,
});
