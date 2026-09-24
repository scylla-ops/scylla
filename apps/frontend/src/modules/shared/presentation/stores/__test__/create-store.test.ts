import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createStore } from '../create-store.ts';

interface CounterState {
  count: number;
  label: string;
  increment: () => void;
}

const createCounter = (persistAs?: string) =>
  createStore<CounterState>(
    (set, get) => ({
      count: 0,
      label: 'counter',
      increment: () => set({ count: get().count + 1 }),
    }),
    { persistAs },
  );

beforeEach(() => localStorage.clear());

describe('createStore', () => {
  it('starts from the state that the initializer returns', () => {
    expect(createCounter().getState().count).toBe(0);
  });

  it('merges a partial state into the current state', () => {
    const store = createCounter();
    store.setState({ count: 3 });

    expect(store.getState()).toMatchObject({ count: 3, label: 'counter' });
  });

  it('accepts a function of the current state', () => {
    const store = createCounter();
    store.setState(state => ({ count: state.count + 2 }));

    expect(store.getState().count).toBe(2);
  });

  it('lets an action write through the setter it received', () => {
    const store = createCounter();
    store.getState().increment();

    expect(store.getState().count).toBe(1);
  });

  it('calls a listener with the new and the previous state', () => {
    const store = createCounter();
    const listener = vi.fn();
    store.subscribe(listener);

    store.setState({ count: 5 });

    expect(listener).toHaveBeenCalledWith(
      expect.objectContaining({ count: 5 }),
      expect.objectContaining({ count: 0 }),
    );
  });

  it('stops calling a listener after it unsubscribes', () => {
    const store = createCounter();
    const listener = vi.fn();
    store.subscribe(listener)();

    store.setState({ count: 5 });

    expect(listener).not.toHaveBeenCalled();
  });

  it('keeps the data fields of a persisted store in localStorage', () => {
    createCounter('counter-key').setState({ count: 4 });

    expect(JSON.parse(localStorage.getItem('counter-key') ?? '')).toEqual({
      state: { count: 4, label: 'counter' },
    });
  });

  it('restores a persisted store and keeps its actions', () => {
    localStorage.setItem('counter-key', JSON.stringify({ state: { count: 7 }, version: 0 }));
    const store = createCounter('counter-key');

    expect(store.getState().count).toBe(7);
    store.getState().increment();
    expect(store.getState().count).toBe(8);
  });

  it('starts from the initial state when the stored value is not valid JSON', () => {
    localStorage.setItem('counter-key', '{not json');

    expect(createCounter('counter-key').getState().count).toBe(0);
  });
});
