import { describe, it, expect, afterEach, beforeEach, vi } from 'vitest';
import { flushSync } from 'svelte';
import { createNow } from '../now.svelte.ts';

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('createNow', () => {
  it('ticks while enabled', () => {
    const cleanup = $effect.root(() => {
      const now = createNow();
      flushSync();
      const first = now.value;

      vi.advanceTimersByTime(3000);
      flushSync();

      expect(now.value).toBeGreaterThan(first);
    });

    cleanup();
  });

  it('stops when the caller says the thing it was timing has finished', () => {
    const cleanup = $effect.root(() => {
      // A getter, not a boolean: this is the whole reason for the signature.
      // Captured once, a job that finishes would leave the timer running.
      let running = $state(true);
      const now = createNow(() => running);
      flushSync();

      vi.advanceTimersByTime(2000);
      flushSync();
      const whileRunning = now.value;

      running = false;
      flushSync();
      vi.advanceTimersByTime(10_000);
      flushSync();

      expect(now.value).toBe(whileRunning);
    });

    cleanup();
  });

  it('clears its interval when the component goes away', () => {
    const clearInterval = vi.spyOn(window, 'clearInterval');

    const cleanup = $effect.root(() => {
      createNow();
      flushSync();
    });
    cleanup();

    expect(clearInterval).toHaveBeenCalled();
  });
});
