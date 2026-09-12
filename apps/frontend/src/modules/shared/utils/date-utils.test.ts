import { describe, it, expect, vi, afterEach } from 'vitest';
import {
  calculateDuration,
  calculateExecutionDuration,
  formatDuration,
  getRelativeTime,
  formatDate,
  formatDay,
  formatTime,
} from './date-utils';

describe('calculateDuration', () => {
  it('returns the elapsed seconds between two ISO timestamps', () => {
    expect(calculateDuration('2026-01-01T00:00:00.000Z', '2026-01-01T00:01:30.000Z')).toBe(90);
  });

  it('returns 0 for identical timestamps', () => {
    expect(calculateDuration('2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z')).toBe(0);
  });
});

describe('calculateExecutionDuration', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns null while the node has not started yet', () => {
    expect(calculateExecutionDuration(undefined, undefined)).toBeNull();
  });

  it('counts up to now while running (no finishedAt)', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:05:00.000Z'));
    expect(calculateExecutionDuration('2026-01-01T00:00:00.000Z')).toBe(300);
  });

  it('uses finishedAt once the node is done', () => {
    expect(
      calculateExecutionDuration('2026-01-01T00:00:00.000Z', '2026-01-01T00:00:45.000Z'),
    ).toBe(45);
  });

  it('never goes negative if the clock is off (finishedAt before startedAt)', () => {
    expect(
      calculateExecutionDuration('2026-01-01T00:00:45.000Z', '2026-01-01T00:00:00.000Z'),
    ).toBe(0);
  });
});

describe('formatDuration', () => {
  it('formats seconds only under a minute', () => {
    expect(formatDuration(45)).toBe('45s');
  });

  it('formats minutes and seconds under an hour', () => {
    expect(formatDuration(72)).toBe('1m 12s');
  });

  it('formats hours and minutes, dropping seconds, at an hour or more', () => {
    expect(formatDuration(3661)).toBe('1h 1m');
  });

  it('formats exactly 0 seconds', () => {
    expect(formatDuration(0)).toBe('0s');
  });
});

describe('getRelativeTime', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns "-" for an unparseable date string', () => {
    expect(getRelativeTime('not-a-date')).toBe('-');
  });

  it('reports the current instant as "now"', () => {
    vi.useFakeTimers();
    const now = new Date('2026-01-01T00:00:00.000Z');
    vi.setSystemTime(now);
    expect(getRelativeTime(now.toISOString())).toBe('now');
  });

  it('picks the largest applicable unit: days', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-03T00:00:00.000Z'));
    expect(getRelativeTime('2026-01-01T00:00:00.000Z')).toBe('2 days ago');
  });

  it('picks the largest applicable unit: hours', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T03:00:00.000Z'));
    expect(getRelativeTime('2026-01-01T00:00:00.000Z')).toBe('3 hr. ago');
  });

  it('picks the largest applicable unit: minutes', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:05:00.000Z'));
    expect(getRelativeTime('2026-01-01T00:00:00.000Z')).toBe('5 min. ago');
  });

  it('picks the largest applicable unit: seconds', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-01-01T00:00:30.000Z'));
    expect(getRelativeTime('2026-01-01T00:00:00.000Z')).toBe('30 sec. ago');
  });
});

describe('formatDate', () => {
  it('returns "-" for undefined input', () => {
    expect(formatDate(undefined)).toBe('-');
  });

  it('returns "-" for an unparseable string', () => {
    expect(formatDate('not-a-date')).toBe('-');
  });

  it('formats a valid ISO string to a non-empty, locale-formatted string', () => {
    const result = formatDate('2026-01-01T12:00:00.000Z');
    expect(result).not.toBe('-');
    expect(result.length).toBeGreaterThan(0);
  });
});

describe('formatDay', () => {
  it('returns "-" for undefined input', () => {
    expect(formatDay(undefined)).toBe('-');
  });

  it('returns "-" for an unparseable string', () => {
    expect(formatDay('not-a-date')).toBe('-');
  });

  it('formats a valid ISO string to a date-only string, without the time of day', () => {
    const result = formatDay('2026-01-01T23:45:00.000Z');
    expect(result).not.toBe('-');
    expect(result).not.toMatch(/\d{1,2}:\d{2}/);
  });
});

describe('formatTime', () => {
  it('returns "-" for undefined input', () => {
    expect(formatTime(undefined)).toBe('-');
  });

  it('returns "-" for an unparseable string', () => {
    expect(formatTime('not-a-date')).toBe('-');
  });

  it('formats a valid ISO string to a non-empty time-of-day string', () => {
    const result = formatTime('2026-01-01T12:34:56.000Z');
    expect(result).not.toBe('-');
    expect(result.length).toBeGreaterThan(0);
  });
});
