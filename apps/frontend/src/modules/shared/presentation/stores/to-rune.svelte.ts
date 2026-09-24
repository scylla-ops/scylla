import { createSubscriber } from 'svelte/reactivity';

interface ReadableStore<TState> {
  getState: () => TState;
  subscribe: (listener: (state: TState, previous: TState) => void) => () => void;
}

/**
 * Reads a store from rune code. Subscribes only while something reads it; outside
 * a reactive context it returns `getState()`.
 */
export const toRune = <TState>(store: ReadableStore<TState>): (() => TState) => {
  const subscribe = createSubscriber(update => store.subscribe(() => update()));

  return () => {
    subscribe();
    return store.getState();
  };
};
