import { describe, it, expect, beforeEach, vi } from 'vitest';
import { flushSync } from 'svelte';
import { createCompactContainer } from '../compact-container.svelte.ts';

/** Drives the observer the action installs, which `setup.ts` stubs out. */
let notify: ((entries: Array<{ contentRect: { width: number } }>) => void) | null = null;
const disconnect = vi.fn();

beforeEach(() => {
  notify = null;
  disconnect.mockClear();

  vi.stubGlobal(
    'ResizeObserver',
    class {
      constructor(callback: (entries: Array<{ contentRect: { width: number } }>) => void) {
        notify = callback;
      }
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = disconnect;
    },
  );
});

const attach = (container: ReturnType<typeof createCompactContainer>) => {
  const node = document.createElement('div');
  return container.measure(node) as { destroy?: () => void } | undefined;
};

describe('createCompactContainer', () => {
  it('starts roomy, before anything has been measured', () => {
    const cleanup = $effect.root(() => {
      const container = createCompactContainer();
      expect(container.isCompact).toBe(false);
    });
    cleanup();
  });

  it('reports compact once the element is narrower than the threshold', () => {
    const cleanup = $effect.root(() => {
      const container = createCompactContainer(70);
      attach(container);

      notify?.([{ contentRect: { width: 40 } }]);
      flushSync();
      expect(container.isCompact).toBe(true);

      // And back: a column widened by the user must restore the inline layout.
      notify?.([{ contentRect: { width: 200 } }]);
      flushSync();
      expect(container.isCompact).toBe(false);
    });
    cleanup();
  });

  it('measures against the threshold it was given, not a fixed one', () => {
    const cleanup = $effect.root(() => {
      const container = createCompactContainer(500);
      attach(container);

      notify?.([{ contentRect: { width: 300 } }]);
      flushSync();

      expect(container.isCompact).toBe(true);
    });
    cleanup();
  });

  it('disconnects the observer when its element is destroyed', () => {
    const cleanup = $effect.root(() => {
      const container = createCompactContainer();
      attach(container)?.destroy?.();
    });
    cleanup();

    expect(disconnect).toHaveBeenCalled();
  });
});
