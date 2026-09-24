import { createSubscriber } from 'svelte/reactivity';

/** The shape of a store from `createStore`, reduced to what a reader needs. */
interface ReadableStore<TState> {
  getState: () => TState;
  subscribe: (listener: (state: TState, previous: TState) => void) => () => void;
}

/**
 * Reads a store from rune code.
 *
 * The returned function tracks the reactive context that reads it, and the
 * subscription to the store lasts only while something reads it. Outside a
 * reactive context it returns `getState()`, so a helper built on it stays
 * testable in plain TypeScript.
 *
 * ```ts
 * const read = toRune(selectionStore);
 * get selectedIds() { return read().selectedIds[key] ?? EMPTY; }
 * ```
 */
export const toRune = <TState>(store: ReadableStore<TState>): (() => TState) => {
  const subscribe = createSubscriber(update => store.subscribe(() => update()));

  return () => {
    subscribe();
    return store.getState();
  };
};
