/**
 * Whether the user has asked the system for less motion.
 *
 * `index.css` already neutralises the CSS animations under
 * `@media (prefers-reduced-motion: reduce)`, but a Svelte transition is driven
 * from JavaScript: it writes inline styles frame by frame and that media query
 * never sees it. Every transition in this folder therefore asks here and
 * collapses its duration to zero — same preference, honoured twice because
 * there are two mechanisms.
 *
 * Reads the query on each call rather than caching: the preference can change
 * while the app is open, and the answer is only ever needed at the instant a
 * transition starts.
 */
export const prefersReducedMotion = (): boolean =>
  typeof window !== 'undefined' &&
  typeof window.matchMedia === 'function' &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches;

/** `duration`, or 0 when the user asked for less motion. */
export const motionDuration = (duration: number): number =>
  prefersReducedMotion() ? 0 : duration;
