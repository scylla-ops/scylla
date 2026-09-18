import { createSubscriber } from 'svelte/reactivity';

/** The shape Zustand's `create()` returns, reduced to what a reader needs. */
interface ReadableStore<TState> {
  getState: () => TState;
  subscribe: (listener: (state: TState, previous: TState) => void) => () => void;
}

/**
 * Reads a Zustand store from rune code.
 *
 * `to-svelte-store.ts` next door answers a different question: it produces the
 * `subscribe` contract a `.svelte` template consumes with `$store`. That is not
 * usable from a `.svelte.ts` module, where there is no `$`-prefix and often no
 * component instance to own an `$effect`.
 *
 * `createSubscriber` is Svelte's own answer for exactly this. The returned
 * function tracks whatever reactive context reads it and hands back a `update`
 * callback to fire on change; the subscription starts when something is
 * actually reading and stops when nothing is, so a `.svelte.ts` helper does not
 * leak a listener per call. Outside a reactive context the read simply falls
 * through to `getState()`, which is what makes these helpers testable in plain
 * TypeScript.
 *
 * ```ts
 * const read = toRune(useSelectionStore);
 * get selectedIds() { return read().selectedIds[key] ?? EMPTY; }
 * ```
 *
 * Temporary by construction: Zustand goes away with React in Phase 6 and these
 * stores become plain runes.
 */
export const toRune = <TState>(store: ReadableStore<TState>): (() => TState) => {
  const subscribe = createSubscriber(update => store.subscribe(() => update()));

  return () => {
    subscribe();
    return store.getState();
  };
};
