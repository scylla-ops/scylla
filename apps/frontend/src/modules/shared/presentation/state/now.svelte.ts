/**
 * A ticking timestamp, for the components that show elapsed time while
 * something is still running — the Svelte counterpart of `use-now.ts`.
 *
 * `enabled` is a getter rather than a value: a job that finishes while the page
 * is open must stop the timer, and taking the boolean once would leave it
 * ticking for the lifetime of the component. React re-ran the hook on every
 * render and got this for free; the same trap as `createFeatureSelection`'s id
 * list, from the same cause.
 */
export interface Now {
  /** `Date.now()`, refreshed every `intervalMs` while enabled. */
  readonly value: number;
}

export const createNow = (enabled: () => boolean = () => true, intervalMs = 1000): Now => {
  let now = $state(Date.now());

  // A timer is a system outside Svelte, which is exactly what `$effect` is for
  // — and unlike React's, this one re-runs on its own when `enabled` flips,
  // with no dependency array to get wrong.
  $effect(() => {
    if (!enabled()) return;

    const id = window.setInterval(() => (now = Date.now()), intervalMs);
    return () => window.clearInterval(id);
  });

  return {
    get value() {
      return now;
    },
  };
};
