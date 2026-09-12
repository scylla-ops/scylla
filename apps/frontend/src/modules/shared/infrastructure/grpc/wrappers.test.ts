import { describe, it, expect } from 'vitest';
import { idValue, wrapId, wrapIdOpt, timestampToIso, timestampToIsoOpt } from './wrappers';

describe('idValue', () => {
  it('unwraps a present id message', () => {
    expect(idValue({ value: 'abc' })).toBe('abc');
  });

  it('falls back to an empty string when the wrapper is absent', () => {
    expect(idValue(undefined)).toBe('');
  });
});

describe('wrapId', () => {
  it('wraps a plain string into a {value} message', () => {
    expect(wrapId('abc')).toEqual({ value: 'abc' });
  });
});

describe('wrapIdOpt', () => {
  it('wraps a non-empty string', () => {
    expect(wrapIdOpt('abc')).toEqual({ value: 'abc' });
  });

  it('returns undefined for an empty string (preserves proto3 field-absence semantics)', () => {
    expect(wrapIdOpt('')).toBeUndefined();
  });

  it('returns undefined when given undefined', () => {
    expect(wrapIdOpt(undefined)).toBeUndefined();
  });
});

describe('timestampToIso', () => {
  it('formats seconds+nanos to a UTC ISO string', () => {
    expect(timestampToIso({ seconds: 1735689600, nanos: 146000000 })).toBe('2025-01-01T00:00:00.146Z');
  });

  it('accepts seconds as a string or a bigint, with the same result', () => {
    const expected = timestampToIso({ seconds: 1735689600, nanos: 0 });
    expect(timestampToIso({ seconds: '1735689600', nanos: 0 })).toBe(expected);
    expect(timestampToIso({ seconds: 1735689600n, nanos: 0 })).toBe(expected);
  });

  it('returns an empty string when the timestamp is absent', () => {
    expect(timestampToIso(undefined)).toBe('');
  });

  it('rounds nanos down to millisecond precision', () => {
    // 999_999 ns is under 1ms and must not round up into the next second.
    expect(timestampToIso({ seconds: 0, nanos: 999_999 })).toBe('1970-01-01T00:00:00.000Z');
  });
});

describe('timestampToIsoOpt', () => {
  it('returns undefined when the timestamp is absent', () => {
    expect(timestampToIsoOpt(undefined)).toBeUndefined();
  });

  it('returns undefined for the zero timestamp (treated as "not started/finished yet")', () => {
    expect(timestampToIsoOpt({ seconds: 0, nanos: 0 })).toBeUndefined();
  });

  it('returns undefined for the zero timestamp given as a string or bigint zero', () => {
    expect(timestampToIsoOpt({ seconds: '0', nanos: 0 })).toBeUndefined();
    expect(timestampToIsoOpt({ seconds: 0n, nanos: 0 })).toBeUndefined();
  });

  it('formats a genuinely set timestamp', () => {
    expect(timestampToIsoOpt({ seconds: 1735689600, nanos: 0 })).toBe('2025-01-01T00:00:00.000Z');
  });
});
