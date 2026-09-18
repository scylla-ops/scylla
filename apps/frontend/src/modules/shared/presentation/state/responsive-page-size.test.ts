// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { computePageSize } from './responsive-page-size.ts';

describe('computePageSize', () => {
  it('fits as many rows as the room left under the header allows', () => {
    expect(computePageSize(640, 60, 40)).toBe(10);
    expect(computePageSize(1240, 60, 40)).toBe(20);
  });

  it('never goes below the floor, however little room there is', () => {
    expect(computePageSize(0, 60, 40)).toBe(5);
    expect(computePageSize(-500, 60, 40)).toBe(5);
  });

  it('never goes above the ceiling, however much room there is', () => {
    expect(computePageSize(100_000, 60, 40)).toBe(50);
  });

  it('falls back to the default row and header heights', () => {
    // 61px rows under a 44px header: (700 - 44) / 61 -> 10
    expect(computePageSize(700)).toBe(10);
  });
});
