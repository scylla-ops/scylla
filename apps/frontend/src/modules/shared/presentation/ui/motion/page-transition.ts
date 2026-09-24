import { cubicOut, cubicIn } from 'svelte/easing';
import type { TransitionConfig } from 'svelte/transition';
import { motionDuration } from './reduced-motion.ts';

/**
 * How long the departing page takes to fade out. The arriving page stays fully
 * transparent for this long, so the text of the two pages never shows at the
 * same time.
 */
export const PAGE_OUT_MS = 100;

/**
 * How long the arriving page takes to settle once the departing page is gone.
 * Together with `PAGE_OUT_MS` the whole change stays under the 400 ms that
 * framer-motion's `mode='wait'` took.
 */
export const PAGE_IN_MS = 200;

const PAGE_CHANGE_MS = PAGE_OUT_MS + PAGE_IN_MS;

/**
 * The arriving page: waits out `PAGE_OUT_MS` fully transparent, then fades up
 * from a hair under full size.
 *
 * The wait is part of the curve rather than a `delay`: the page is inserted
 * at once, and a delayed transition would show it at full opacity until the
 * delay ends.
 *
 * The unused `node` parameter is the transition contract — Svelte calls
 * `in:pageIn` as `pageIn(node, params)`, and a zero-argument signature is
 * rejected by `svelte-check` even though it would run fine.
 */
export const pageIn = (_node?: Element): TransitionConfig => ({
  duration: motionDuration(PAGE_CHANGE_MS),
  css: t => {
    const progress = cubicOut(Math.max(0, (t * PAGE_CHANGE_MS - PAGE_OUT_MS) / PAGE_IN_MS));
    return `opacity: ${progress}; transform: scale(${0.99 + 0.01 * progress})`;
  },
});

/**
 * The departing page: fades out while easing very slightly *away* from the
 * viewer, and is gone before the arriving page starts to show.
 *
 * The node must be taken out of flow while this runs, or it pushes the arriving
 * page down for the duration — `PageTransition.svelte` is where that happens.
 */
export const pageOut = (_node?: Element): TransitionConfig => ({
  duration: motionDuration(PAGE_OUT_MS),
  easing: cubicIn,
  css: (t, u) => `opacity: ${t}; transform: scale(${1 + 0.006 * u})`,
});
