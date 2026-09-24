export type StoreListener<TState> = (state: TState, previous: TState) => void;

export type SetState<TState> = (
  partial: Partial<TState> | ((state: TState) => Partial<TState>),
) => void;

export interface Store<TState> {
  getState: () => TState;
  setState: SetState<TState>;
  subscribe: (listener: StoreListener<TState>) => () => void;
}

export interface StoreOptions {
  /** Keeps the data fields in `localStorage` under this key. */
  persistAs?: string;
}

const dataOf = <TState extends object>(state: TState): Partial<TState> =>
  Object.fromEntries(
    Object.entries(state).filter(([, value]) => typeof value !== 'function'),
  ) as Partial<TState>;

const storage = (): Storage | undefined =>
  typeof localStorage === 'undefined' ? undefined : localStorage;

const restore = <TState extends object>(key: string): Partial<TState> => {
  try {
    const stored = JSON.parse(storage()?.getItem(key) ?? 'null') as { state?: Partial<TState> };
    return stored?.state ?? {};
  } catch {
    return {};
  }
};

/** `setState` merges the partial state and calls every listener. Read it from runes with `toRune`. */
export const createStore = <TState extends object>(
  initializer: (set: SetState<TState>, get: () => TState) => TState,
  { persistAs }: StoreOptions = {},
): Store<TState> => {
  const listeners = new Set<StoreListener<TState>>();

  const getState = () => state;

  const setState: SetState<TState> = partial => {
    const previous = state;
    const next = typeof partial === 'function' ? partial(state) : partial;
    state = { ...state, ...next };
    if (persistAs) storage()?.setItem(persistAs, JSON.stringify({ state: dataOf(state) }));
    listeners.forEach(listener => listener(state, previous));
  };

  let state: TState = initializer(setState, getState);
  if (persistAs) state = { ...state, ...restore<TState>(persistAs) };

  return {
    getState,
    setState,
    subscribe: listener => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
};
