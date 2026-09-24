import { describe, it, expect, vi, afterEach } from 'vitest';
import { PAGE_IN_MS, PAGE_OUT_MS, pageIn, pageOut } from '../page-transition.ts';
import { motionDuration, prefersReducedMotion } from '../reduced-motion.ts';

const setReducedMotion = (reduce: boolean) => {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches: reduce && query === '(prefers-reduced-motion: reduce)',
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
};

afterEach(() => vi.restoreAllMocks());

describe('prefersReducedMotion', () => {
  it('is false when the user has expressed no preference', () => {
    setReducedMotion(false);

    expect(prefersReducedMotion()).toBe(false);
  });

  it('is true when the user asked for less motion', () => {
    setReducedMotion(true);

    expect(prefersReducedMotion()).toBe(true);
  });

  it('collapses any duration to zero under the preference', () => {
    setReducedMotion(true);

    expect(motionDuration(400)).toBe(0);
  });
});

describe('page transitions', () => {
  const inAt = (ms: number) => pageIn().css?.(ms / (PAGE_OUT_MS + PAGE_IN_MS), 0) ?? '';

  it('removes the outgoing page before the incoming page starts to show', () => {
    setReducedMotion(false);

    expect(pageOut().duration).toBe(PAGE_OUT_MS);
    expect(inAt(0)).toContain('opacity: 0;');
    expect(inAt(PAGE_OUT_MS)).toContain('opacity: 0;');
    expect(inAt(PAGE_OUT_MS + PAGE_IN_MS / 2)).not.toContain('opacity: 0;');
  });

  it('settles the incoming page within the 400 ms that the React transition took', () => {
    setReducedMotion(false);

    expect(pageIn().duration).toBe(PAGE_OUT_MS + PAGE_IN_MS);
    expect(PAGE_OUT_MS + PAGE_IN_MS).toBeLessThan(400);
  });

  it('fades the arriving page up from fully transparent to fully opaque', () => {
    setReducedMotion(false);

    expect(inAt(0)).toContain('opacity: 0');
    expect(inAt(PAGE_OUT_MS + PAGE_IN_MS)).toContain('opacity: 1');
  });

  it('scales the arriving page by a hair only — a visible zoom reads as a rebuild', () => {
    setReducedMotion(false);

    expect(inAt(0)).toContain('scale(0.99)');
    expect(inAt(PAGE_OUT_MS + PAGE_IN_MS)).toContain('scale(1)');
  });

  it('honours prefers-reduced-motion, which a CSS media query cannot do for a JS transition', () => {
    setReducedMotion(true);

    expect(pageIn().duration).toBe(0);
    expect(pageOut().duration).toBe(0);
  });
});
