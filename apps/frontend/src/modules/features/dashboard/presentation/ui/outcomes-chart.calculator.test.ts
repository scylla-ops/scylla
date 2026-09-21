// @vitest-environment node
import { describe, it, expect } from 'vitest';
import {
  fillBuckets,
  hasActivity,
  labelVisibility,
  localDay,
  monotoneAreaPath,
  monotoneLinePath,
  niceTicks,
  peakOf,
  projectPoints,
  type ChartGeometry,
  type Point,
} from './outcomes-chart.calculator.ts';

const GEOMETRY: ChartGeometry = {
  width: 100,
  height: 100,
  padding: { top: 0, right: 0, bottom: 0, left: 0 },
};

/** Samples a cubic bezier segment, so the curve can be checked between knots. */
const sampleCubic = (p0: Point, c1: Point, c2: Point, p1: Point, steps = 20): number[] =>
  Array.from({ length: steps + 1 }, (_, i) => {
    const t = i / steps;
    const u = 1 - t;
    return u * u * u * p0.y + 3 * u * u * t * c1.y + 3 * u * t * t * c2.y + t * t * t * p1.y;
  });

/** Pulls the numbers back out of a `d` string, in order. */
const numbersIn = (path: string): number[] =>
  (path.match(/-?\d+(\.\d+)?/g) ?? []).map(Number);

describe('localDay', () => {
  it('formats a local calendar day, zero-padded', () => {
    expect(localDay(new Date(2026, 0, 5))).toBe('2026-01-05');
    expect(localDay(new Date(2026, 11, 31))).toBe('2026-12-31');
  });
});

describe('fillBuckets', () => {
  const today = new Date(2026, 2, 10);

  it('returns one bucket per day of the window, oldest first', () => {
    const buckets = fillBuckets([], 3, today);

    expect(buckets.map(bucket => bucket.day)).toEqual(['2026-03-08', '2026-03-09', '2026-03-10']);
  });

  it('treats a day the backend never sent as a real zero, not a gap', () => {
    // Drawing straight across the hole would claim activity that never happened.
    const buckets = fillBuckets(
      [{ day: new Date(2026, 2, 10).toISOString(), completed: 4, failed: 1, cancelled: 0 }],
      3,
      today,
    );

    expect(buckets[0]).toMatchObject({ completed: 0, failed: 0, cancelled: 0 });
    expect(buckets[2]).toMatchObject({ completed: 4, failed: 1, cancelled: 0 });
  });

  it('labels each bucket with its day of the month', () => {
    expect(fillBuckets([], 3, today).map(bucket => bucket.label)).toEqual(['8', '9', '10']);
  });

  it('crosses a month boundary rather than counting backwards within one', () => {
    expect(fillBuckets([], 3, new Date(2026, 2, 1)).map(bucket => bucket.day)).toEqual([
      '2026-02-27',
      '2026-02-28',
      '2026-03-01',
    ]);
  });

  it('never returns fewer buckets than the window asked for', () => {
    expect(fillBuckets([], 30, today)).toHaveLength(30);
  });
});

describe('hasActivity', () => {
  it('is false for a window where nothing finished', () => {
    expect(hasActivity(fillBuckets([], 7, new Date(2026, 2, 10)))).toBe(false);
  });

  it('is true as soon as one cancelled run exists', () => {
    const buckets = fillBuckets(
      [{ day: new Date(2026, 2, 10).toISOString(), completed: 0, failed: 0, cancelled: 1 }],
      7,
      new Date(2026, 2, 10),
    );

    expect(hasActivity(buckets)).toBe(true);
  });
});

describe('peakOf', () => {
  const buckets = fillBuckets(
    [
      { day: new Date(2026, 2, 9).toISOString(), completed: 9, failed: 0, cancelled: 0 },
      { day: new Date(2026, 2, 10).toISOString(), completed: 1, failed: 4, cancelled: 0 },
    ],
    2,
    new Date(2026, 2, 10),
  );

  it('measures across every series being drawn', () => {
    expect(peakOf(buckets, ['completed', 'failed', 'cancelled'])).toBe(9);
  });

  it('ignores a series the filter has hidden — the axis follows what is shown', () => {
    expect(peakOf(buckets, ['failed'])).toBe(4);
  });

  it('is zero for an empty window', () => {
    expect(peakOf([], ['completed'])).toBe(0);
  });
});

describe('niceTicks', () => {
  it('counts whole runs only — half a job is not a quantity', () => {
    for (const peak of [1, 3, 7, 12, 47, 133]) {
      expect(niceTicks(peak).every(Number.isInteger)).toBe(true);
    }
  });

  it('always starts at zero and covers the peak', () => {
    for (const peak of [1, 3, 7, 12, 47, 133]) {
      const ticks = niceTicks(peak);
      expect(ticks[0]).toBe(0);
      expect(ticks[ticks.length - 1]).toBeGreaterThanOrEqual(peak);
    }
  });

  it('is strictly increasing with an even step', () => {
    const ticks = niceTicks(47);
    const step = ticks[1] - ticks[0];
    expect(step).toBeGreaterThan(0);
    ticks.forEach((tick, index) => expect(tick).toBe(index * step));
  });

  it('gives an empty chart a readable axis rather than a single line', () => {
    expect(niceTicks(0)).toEqual([0, 1]);
    expect(niceTicks(-5)).toEqual([0, 1]);
    expect(niceTicks(Number.NaN)).toEqual([0, 1]);
  });
});

describe('projectPoints', () => {
  it('spreads the buckets evenly across the full width', () => {
    const points = projectPoints([0, 0, 0], 1, GEOMETRY);

    expect(points.map(point => point.x)).toEqual([0, 50, 100]);
  });

  it('puts the biggest count at the top, since SVG y grows downward', () => {
    const points = projectPoints([0, 5], 5, GEOMETRY);

    expect(points[0].y).toBe(100);
    expect(points[1].y).toBe(0);
  });

  it('honours the padding it is given', () => {
    const points = projectPoints([0, 0], 1, {
      width: 100,
      height: 100,
      padding: { top: 10, right: 10, bottom: 10, left: 20 },
    });

    expect(points[0].x).toBe(20);
    expect(points[1].x).toBe(90);
    expect(points[0].y).toBe(90);
  });

  it('does not divide by zero when nothing ever ran', () => {
    const points = projectPoints([0, 0], 0, GEOMETRY);

    expect(points.every(point => Number.isFinite(point.y))).toBe(true);
  });

  it('survives a single bucket', () => {
    expect(projectPoints([3], 3, GEOMETRY)).toEqual([{ x: 0, y: 0 }]);
  });
});

describe('monotoneLinePath', () => {
  it('is empty for no points rather than a malformed path', () => {
    expect(monotoneLinePath([])).toBe('');
  });

  it('is a bare move for a single point', () => {
    expect(monotoneLinePath([{ x: 1, y: 2 }])).toBe('M1,2');
  });

  it('starts at the first point and ends at the last', () => {
    const points = projectPoints([0, 3, 1, 4], 4, GEOMETRY);
    const path = monotoneLinePath(points);

    expect(path.startsWith(`M${points[0].x},${points[0].y}`)).toBe(true);
    const numbers = numbersIn(path);
    expect(numbers[numbers.length - 2]).toBeCloseTo(points[3].x, 1);
    expect(numbers[numbers.length - 1]).toBeCloseTo(points[3].y, 1);
  });

  it('emits one cubic segment per gap', () => {
    const points = projectPoints([0, 1, 2, 3], 3, GEOMETRY);

    expect(monotoneLinePath(points).match(/C/g)).toHaveLength(3);
  });

  it('never overshoots below zero between two days with no runs', () => {
    // The whole reason the curve is monotone: a plain spline dips under the
    // baseline here and draws negative jobs.
    const values = [0, 0, 5, 0, 0];
    const points = projectPoints(values, 5, GEOMETRY);
    const tangentsPath = monotoneLinePath(points);
    const numbers = numbersIn(tangentsPath);

    // Walk the emitted control points and sample every segment.
    let cursor = 2;
    for (let i = 0; i < points.length - 1; i += 1) {
      const c1 = { x: numbers[cursor], y: numbers[cursor + 1] };
      const c2 = { x: numbers[cursor + 2], y: numbers[cursor + 3] };
      const end = { x: numbers[cursor + 4], y: numbers[cursor + 5] };
      cursor += 6;

      for (const y of sampleCubic(points[i], c1, c2, end)) {
        // y is in SVG space: the baseline is the largest y.
        expect(y).toBeLessThanOrEqual(GEOMETRY.height + 0.01);
        expect(y).toBeGreaterThanOrEqual(-0.01);
      }
    }
  });

  it('stays inside the interval its endpoints define, for any run pattern', () => {
    const patterns = [
      [0, 9, 0, 9, 0],
      [3, 3, 3, 3],
      [0, 1, 2, 10, 2, 1, 0],
      [7, 0, 0, 0, 7],
    ];

    for (const values of patterns) {
      const points = projectPoints(values, Math.max(...values), GEOMETRY);
      const numbers = numbersIn(monotoneLinePath(points));

      let cursor = 2;
      for (let i = 0; i < points.length - 1; i += 1) {
        const c1 = { x: numbers[cursor], y: numbers[cursor + 1] };
        const c2 = { x: numbers[cursor + 2], y: numbers[cursor + 3] };
        const end = { x: numbers[cursor + 4], y: numbers[cursor + 5] };
        cursor += 6;

        const low = Math.min(points[i].y, end.y) - 0.01;
        const high = Math.max(points[i].y, end.y) + 0.01;
        for (const y of sampleCubic(points[i], c1, c2, end)) {
          expect(y).toBeGreaterThanOrEqual(low);
          expect(y).toBeLessThanOrEqual(high);
        }
      }
    }
  });

  it('draws a flat run of equal days as a flat line', () => {
    const points = projectPoints([2, 2, 2], 2, GEOMETRY);
    const numbers = numbersIn(monotoneLinePath(points));

    expect(numbers.filter((_, index) => index % 2 === 1).every(y => y === 0)).toBe(true);
  });
});

describe('monotoneAreaPath', () => {
  it('is empty when there is nothing to draw', () => {
    expect(monotoneAreaPath([], 100)).toBe('');
  });

  it('closes the curve down to the baseline', () => {
    const points = projectPoints([1, 2], 2, GEOMETRY);
    const path = monotoneAreaPath(points, 100);

    expect(path.endsWith('Z')).toBe(true);
    expect(path).toContain('L100,100');
    expect(path).toContain('L0,100');
  });

  it('shares its opening with the line, so fill and stroke cannot drift', () => {
    const points = projectPoints([1, 4, 2], 4, GEOMETRY);

    expect(monotoneAreaPath(points, 100).startsWith(monotoneLinePath(points))).toBe(true);
  });
});

describe('labelVisibility', () => {
  it('shows every label for a short window', () => {
    expect(labelVisibility(7)).toEqual(new Array(7).fill(true));
  });

  it('thins a long window rather than smearing 30 numbers together', () => {
    const visible = labelVisibility(30);

    expect(visible.filter(Boolean).length).toBeLessThan(30);
  });

  it('always keeps both ends — they are the window being described', () => {
    for (const count of [14, 30, 90]) {
      const visible = labelVisibility(count);
      expect(visible[0]).toBe(true);
      expect(visible[count - 1]).toBe(true);
    }
  });
});
