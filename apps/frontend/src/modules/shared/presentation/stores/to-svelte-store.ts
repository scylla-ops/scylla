/** The shape Zustand's `create()` returns, reduced to what a reader needs. */
interface ReadableStore<TState> {
  getState: () => TState;
  subscribe: (listener: (state: TState, previous: TState) => void) => () => void;
}

/** Svelte's store contract: `subscribe` calls back immediately, then on change. */
interface SvelteStore<TValue> {
  subscribe: (run: (value: TValue) => void) => () => void;
}

/**
 * Reads a Zustand store from Svelte.
 *
 * The two contracts differ in one detail that matters: Svelte expects
 * `subscribe` to call back **immediately** with the current value, Zustand only
 * calls on change. Without the first call a component renders empty until
 * something happens to the store — which, for the context store, can be never.
 *
 * `selector` is what keeps a component from re-rendering on every unrelated
 * field; the equality check is reference-based, so return a primitive or a
 * stable reference from it, exactly as with `useStore`.
 *
 * Temporary by construction: Zustand goes away with React in Phase 6, and these
 * stores become plain runes. Until then this is the one bridge, rather than one
 * ad-hoc subscription per component.
 */
export const toSvelteStore = <TState, TValue = TState>(
  store: ReadableStore<TState>,
  selector: (state: TState) => TValue = state => state as unknown as TValue,
): SvelteStore<TValue> => ({
  subscribe: run => {
    run(selector(store.getState()));

    let previous = selector(store.getState());

    return store.subscribe(state => {
      const next = selector(state);
      if (next === previous) return;
      previous = next;
      run(next);
    });
  },
});
