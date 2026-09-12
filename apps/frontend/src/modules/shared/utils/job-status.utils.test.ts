import { describe, it, expect } from 'vitest';
import { toStatusState } from './job-status.utils';

describe('toStatusState', () => {
  it.each([
    ['pending', 'idle'],
    ['running', 'running'],
    ['completed', 'success'],
    ['success', 'success'],
    ['failed', 'failed'],
    ['skipped', 'skipped'],
    ['cancelled', 'cancelled'],
  ] as const)('maps raw status %s to %s', (raw, expected) => {
    expect(toStatusState(raw)).toBe(expected);
  });

  it('is case-insensitive (API casing should never break the mapping)', () => {
    expect(toStatusState('RUNNING')).toBe('running');
    expect(toStatusState('Failed')).toBe('failed');
  });

  it('falls back to idle for an unknown status', () => {
    expect(toStatusState('some-future-status')).toBe('idle');
  });

  it('falls back to idle when status is undefined', () => {
    expect(toStatusState(undefined)).toBe('idle');
  });
});
