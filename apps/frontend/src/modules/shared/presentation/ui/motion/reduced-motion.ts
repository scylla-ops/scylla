/**
 * `index.css` handles the CSS animations; a Svelte transition runs in JavaScript
 * and must ask here. Not cached: the preference can change while the app is open.
 */
export const prefersReducedMotion = (): boolean =>
  typeof window !== 'undefined' &&
  typeof window.matchMedia === 'function' &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches;

export const motionDuration = (duration: number): number =>
  prefersReducedMotion() ? 0 : duration;
