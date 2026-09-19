import { useEffect, useState } from 'react';
import type { Component } from 'svelte';
import { SvelteIsland } from './SvelteIsland.tsx';

interface LazySvelteIslandProps<TProps extends Record<string, unknown>> {
  /** `() => import('…/X.svelte')` — a function, so the chunk is a separate one. */
  load: () => Promise<{ default: Component<TProps> }>;
  props: TProps;
}

/**
 * An island whose component is fetched when it is first needed.
 *
 * A **static** import of a `.svelte` file from the shell puts the Svelte
 * runtime and bits-ui in the entry chunk — measured: 56 kB gzip on first paint,
 * for a dialog nobody has opened. Rollup cannot drop a re-exported component
 * from a barrel the shell imports for other reasons, so the deferral has to be
 * a dynamic import, which is what `load` is.
 *
 * Renders nothing until the chunk arrives. That is right for what this is used
 * for — dialogs, mounted only once something is opened — and a spinner would
 * flash for the length of a local fetch.
 *
 * Temporary, like `SvelteIsland` itself: both go in Phase 6.
 */
export const LazySvelteIsland = <TProps extends Record<string, unknown>>({
  load,
  props,
}: LazySvelteIslandProps<TProps>) => {
  const [component, setComponent] = useState<Component<TProps> | null>(null);

  useEffect(() => {
    let alive = true;
    // `setComponent(() => …)`: a component is a function, and the updater form
    // is the only way to store one without React calling it.
    void load().then(module => alive && setComponent(() => module.default));
    return () => {
      alive = false;
    };
  }, [load]);

  return component && <SvelteIsland component={component} props={props} />;
};
