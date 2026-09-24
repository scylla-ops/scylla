/** A ticking timestamp. `enabled` is a getter, so the timer stops when a job finishes. */
export interface Now {
  readonly value: number;
}

export const createNow = (enabled: () => boolean = () => true, intervalMs = 1000): Now => {
  let now = $state(Date.now());

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
