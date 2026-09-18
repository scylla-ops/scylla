import { describe, it, expect, vi, afterEach } from 'vitest';
import { PAGE_IN_MS, PAGE_OUT_MS, pageIn, pageOut } from './page-transition.ts';
import { motionDuration, prefersReducedMotion } from './reduced-motion.ts';

/** `setup.ts` reports "no match" for every query; override the one that matters. */
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
  it('never holds a route back: the arriving page starts at once and settles inside 200ms', () => {
    setReducedMotion(false);
    const config = pageIn();

    // The absence of a delay is the substantive claim — framer's `mode="wait"`
    // was the 200 ms of nothing this port exists to remove.
    expect(config.delay ?? 0).toBe(0);
    expect(config.duration).toBe(PAGE_IN_MS);
    expect(PAGE_IN_MS).toBeLessThanOrEqual(200);
  });

  it('lets the outgoing page leave faster than the incoming one arrives, so the two overlap', () => {
    setReducedMotion(false);

    expect(pageOut().duration).toBe(PAGE_OUT_MS);
    expect(PAGE_OUT_MS).toBeLessThan(PAGE_IN_MS);
  });

  it('fades the arriving page up from fully transparent to fully opaque', () => {
    setReducedMotion(false);
    const { css } = pageIn();

    expect(css?.(0, 1)).toContain('opacity: 0');
    expect(css?.(1, 0)).toContain('opacity: 1');
  });

  it('scales the arriving page by a hair only — a visible zoom reads as a rebuild', () => {
    setReducedMotion(false);
    const { css } = pageIn();

    expect(css?.(0, 1)).toContain('scale(0.99)');
    expect(css?.(1, 0)).toContain('scale(1)');
  });

  it('honours prefers-reduced-motion, which a CSS media query cannot do for a JS transition', () => {
    setReducedMotion(true);

    expect(pageIn().duration).toBe(0);
    expect(pageOut().duration).toBe(0);
  });
});
