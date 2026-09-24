// @vitest-environment node
import { describe, it, expect, afterEach, beforeEach, vi } from 'vitest';
import type { DailyOutcome } from '../../../../domain/structs/agent.struct.ts';
import { bucketTotal, bucketsMax, fillBuckets } from '../outcomes-chart.calculator.ts';

const outcome = (day: string, overrides: Partial<DailyOutcome> = {}): DailyOutcome => ({
  day,
  completed: 0,
  failed: 0,
  cancelled: 0,
  orphaned: 0,
  medianDurationMs: null,
  ...overrides,
});

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date('2026-03-10T12:00:00'));
});

afterEach(() => vi.useRealTimers());

describe('fillBuckets', () => {
  it('returns exactly as many days as the range asks for', () => {
    expect(fillBuckets([], '7d')).toHaveLength(7);
    expect(fillBuckets([], '14d')).toHaveLength(14);
    expect(fillBuckets([], '30d')).toHaveLength(30);
  });

  it('ends on today and runs oldest first', () => {
    const buckets = fillBuckets([], '7d');

    expect(buckets[6].day).toBe('2026-03-10');
    expect(buckets[0].day).toBe('2026-03-04');
  });

  it('zero-fills the days the backend left out', () => {
    // A missing day is an empty column, never a shift of the others.
    const buckets = fillBuckets([outcome('2026-03-10T08:00:00', { completed: 3 })], '7d');

    expect(buckets.filter(bucket => bucketTotal(bucket) === 0)).toHaveLength(6);
    expect(buckets[6]).toMatchObject({ completed: 3, failed: 0, cancelled: 0 });
  });

  it('lands an outcome on its local calendar day, not its UTC one', () => {
    // Late local time is the next UTC day: bucket on the local date.
    const buckets = fillBuckets([outcome('2026-03-10T23:30:00', { failed: 2 })], '7d');

    expect(buckets[6]).toMatchObject({ day: '2026-03-10', failed: 2 });
  });

  it('ignores outcomes older than the window', () => {
    const buckets = fillBuckets([outcome('2026-01-01T00:00:00', { completed: 99 })], '7d');

    expect(buckets.every(bucket => bucketTotal(bucket) === 0)).toBe(true);
  });
});

describe('bucketsMax', () => {
  it('is the tallest stacked column', () => {
    const buckets = fillBuckets(
      [
        outcome('2026-03-09T10:00:00', { completed: 2, failed: 1 }),
        outcome('2026-03-10T10:00:00', { completed: 4, cancelled: 1 }),
      ],
      '7d',
    );

    expect(bucketsMax(buckets)).toBe(5);
  });

  it('never goes below 1, so an empty window still lays out', () => {
    expect(bucketsMax(fillBuckets([], '7d'))).toBe(1);
  });
});
