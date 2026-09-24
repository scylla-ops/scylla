import { describe, it, expect } from 'vitest';
import { screen } from '@testing-library/svelte';
import { render } from '@/test/render.svelte.ts';
import type { JobsSummary } from '@/modules/features/jobs';
import RunActivityCard from './RunActivityCard.svelte';

const summary = (overrides: Partial<JobsSummary> = {}): JobsSummary =>
  ({
    total: 10,
    completed: 6,
    failed: 2,
    cancelled: 1,
    orphaned: 1,
    pending: 0,
    running: 0,
    successRate: 0.6,
    lastRunAt: null,
    ...overrides,
  }) as JobsSummary;

/**
 * The figure shown under one outcome label.
 *
 * Read from the label rather than from the number: two outcomes routinely share
 * a count, and a bare `getByText('1')` cannot tell which tile it found — which
 * is exactly the mix-up that would make a swapped pair of labels pass.
 */
const tileValue = (label: string): string | undefined =>
  screen.getByText(label).parentElement?.firstElementChild?.textContent ?? undefined;

describe('RunActivityCard', () => {
  it('breaks the window down by outcome', () => {
    render(RunActivityCard, { summary: summary(), totalRuns: 10, truncated: false, loading: false });

    expect(tileValue('Completed')).toBe('6');
    expect(tileValue('Failed')).toBe('2');
    expect(tileValue('Cancelled')).toBe('1');
    expect(tileValue('Orphaned')).toBe('1');
  });

  it('says the figures cover a window, not the whole history, when they do', () => {
    render(RunActivityCard, { summary: summary(), totalRuns: 250, truncated: true, loading: false });

    expect(screen.getByText(/over the last 10 of 250 runs/)).toBeInTheDocument();
  });

  it('claims the whole history only when the window is the whole history', () => {
    render(RunActivityCard, { summary: summary(), totalRuns: 10, truncated: false, loading: false });

    expect(screen.getByText(/over all 10 runs/)).toBeInTheDocument();
  });

  it('counts pending and running together as work in flight', () => {
    render(RunActivityCard, {
      summary: summary({ pending: 2, running: 3 }),
      totalRuns: 10,
      truncated: false,
      loading: false,
    });

    expect(screen.getByText('5 in progress')).toBeInTheDocument();
  });

  it('says nothing about work in flight while the window is still loading', () => {
    render(RunActivityCard, {
      summary: summary({ pending: 2, running: 3 }),
      totalRuns: 10,
      truncated: false,
      loading: true,
    });

    expect(screen.queryByText('5 in progress')).not.toBeInTheDocument();
  });

  it('distinguishes an empty history from a loading one', () => {
    render(RunActivityCard, {
      summary: summary({ total: 0, completed: 0, failed: 0, cancelled: 0, orphaned: 0 }),
      totalRuns: 0,
      truncated: false,
      loading: false,
    });

    expect(screen.getByText('No pipeline has run yet.')).toBeInTheDocument();
  });
});
