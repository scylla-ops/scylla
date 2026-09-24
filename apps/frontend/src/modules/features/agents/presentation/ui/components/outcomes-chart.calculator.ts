import type { DailyOutcome } from '../../../domain/structs/agent.struct.ts';

export type OutcomeRange = '7d' | '14d' | '30d';

export const OUTCOME_RANGES: OutcomeRange[] = ['7d', '14d', '30d'];

const RANGE_DAYS: Record<OutcomeRange, number> = { '7d': 7, '14d': 14, '30d': 30 };

export interface Bucket {
  day: string;
  completed: number;
  failed: number;
  cancelled: number;
}

const localDay = (iso: string): string => {
  const date = new Date(iso);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(
    date.getDate(),
  ).padStart(2, '0')}`;
};

/** Zero-fills the last days of the sparse backend series. */
export const fillBuckets = (daily: DailyOutcome[], range: OutcomeRange): Bucket[] => {
  const days = RANGE_DAYS[range];
  const byDay = new Map(daily.map(outcome => [localDay(outcome.day), outcome]));
  const today = new Date();

  return Array.from({ length: days }, (_, index) => {
    const date = new Date(today);
    date.setDate(today.getDate() - (days - 1 - index));
    const hit = byDay.get(localDay(date.toISOString()));

    return {
      day: localDay(date.toISOString()),
      completed: hit?.completed ?? 0,
      failed: hit?.failed ?? 0,
      cancelled: hit?.cancelled ?? 0,
    };
  });
};

export const bucketTotal = (bucket: Bucket): number =>
  bucket.completed + bucket.failed + bucket.cancelled;

/** At least 1, so an all-zero window still lays out. */
export const bucketsMax = (buckets: Bucket[]): number =>
  Math.max(1, ...buckets.map(bucketTotal));
