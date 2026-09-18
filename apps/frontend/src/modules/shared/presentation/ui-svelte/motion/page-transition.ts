import { cubicOut, cubicIn } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';
import { motionDuration } from './reduced-motion.ts';

/**
 * How long the arriving page takes to settle. The *content is legible well
 * before this* — opacity passes 0.8 around 100 ms on a cubic-out curve — and
 * nothing is ever held back waiting for it, which is the whole point of the
 * change: framer-motion's `mode='wait'` used to keep the new page off screen
 * for the 200 ms the old one spent leaving, and that delay was the only part
 * users could actually feel.
 */
export const PAGE_IN_MS = 200;

/**
 * Shorter than the entrance, deliberately. The two overlap, so the outgoing
 * page is a fading backdrop the new one arrives over rather than a step the
 * user waits through.
 */
export const PAGE_OUT_MS = 140;

/**
 * The arriving page: fades up from a hair under full size.
 *
 * The unused `node` parameter is the transition contract — Svelte calls
 * `in:pageIn` as `pageIn(node, params)`, and a zero-argument signature is
 * rejected by `svelte-check` even though it would run fine.
 *
 * 0.99, not 0.95 — at page scale a visible zoom reads as the interface
 * rebuilding itself. What is wanted is the sense that the content was always
 * there and has just come into focus.
 */
export const pageIn = (_node?: Element): TransitionConfig => ({
  duration: motionDuration(PAGE_IN_MS),
  easing: cubicOut,
  css: t => `opacity: ${t}; transform: scale(${0.99 + 0.01 * t})`,
});

/**
 * The departing page: fades out while easing very slightly *away* from the
 * viewer, so the two pages read as layered rather than as one replacing the
 * other in place.
 *
 * The node must be taken out of flow while this runs, or it pushes the arriving
 * page down for the duration — `PageTransition.svelte` is where that happens.
 */
export const pageOut = (_node?: Element): TransitionConfig => ({
  duration: motionDuration(PAGE_OUT_MS),
  easing: cubicIn,
  css: (t, u) => `opacity: ${t}; transform: scale(${1 + 0.006 * u})`,
});
