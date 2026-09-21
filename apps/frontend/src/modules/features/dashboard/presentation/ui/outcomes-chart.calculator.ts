/**
 * The geometry behind the outcomes area chart — scales, monotone curves, ticks.
 *
 * All of it is pure arithmetic on numbers, deliberately: this is the half that
 * survives a change of framework, and it is the half worth testing. `recharts`
 * used to own it (and brought `d3-*` and `victory-vendor` along for 119 kB
 * gzip); what it actually did for this one chart is below, in ~150 lines with
 * no DOM in sight.
 *
 * Tested in `@vitest-environment node` — building a jsdom to check a cubic
 * would be absurd.
 */

/**
 * What one day of the backend's series has to carry for this chart.
 *
 * Structural, not `DailyOutcome` from `agents`: the geometry has no business
 * knowing what else a day of agent statistics holds, and a test here should not
 * have to build fields the curve never reads.
 */
export interface DailyCounts {
  /** An ISO date-time string, which is what the stats endpoint returns. */
  day: string;
  completed: number;
  failed: number;
  cancelled: number;
}

/** One day's worth of outcomes, with the gaps filled in. */
export interface OutcomeBucket {
  /** `YYYY-MM-DD`, local time — the tooltip's label and the bucket's identity. */
  day: string;
  /** The day of the month, which is all the x axis has room for. */
  label: string;
  completed: number;
  failed: number;
  cancelled: number;
}

export interface Point {
  x: number;
  y: number;
}

/** The box the curve is drawn into, in SVG user units. */
export interface ChartGeometry {
  width: number;
  height: number;
  padding: { top: number; right: number; bottom: number; left: number };
}

export type OutcomeSeries = 'completed' | 'failed' | 'cancelled';

/** `YYYY-MM-DD` in **local** time — the user's day, not UTC's. */
export const localDay = (date: Date): string =>
  `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(
    date.getDate(),
  ).padStart(2, '0')}`;

/**
 * One bucket per day over the window, zero-filled.
 *
 * A missing day is a real zero, not a gap: the backend only sends days that had
 * runs, and drawing a line straight across the hole would claim activity that
 * never happened.
 *
 * `today` is a parameter so the result is a function of its inputs — otherwise
 * every test of this would be a test of the system clock.
 */
export const fillBuckets = (
  daily: readonly DailyCounts[],
  days: number,
  today: Date = new Date(),
): OutcomeBucket[] => {
  const byDay = new Map(daily.map(entry => [localDay(new Date(entry.day)), entry]));

  return Array.from({ length: days }, (_, index) => {
    const date = new Date(today);
    date.setDate(today.getDate() - (days - 1 - index));
    const key = localDay(date);
    const hit = byDay.get(key);

    return {
      day: key,
      label: String(date.getDate()),
      completed: hit?.completed ?? 0,
      failed: hit?.failed ?? 0,
      cancelled: hit?.cancelled ?? 0,
    };
  });
};

/** True once anything at all happened in the window. */
export const hasActivity = (buckets: OutcomeBucket[]): boolean =>
  buckets.some(bucket => bucket.completed + bucket.failed + bucket.cancelled > 0);

/** The tallest stack any single day reaches, across the series being drawn. */
export const peakOf = (buckets: OutcomeBucket[], series: readonly OutcomeSeries[]): number =>
  buckets.reduce((peak, bucket) => Math.max(peak, ...series.map(name => bucket[name])), 0);

/**
 * Axis ticks that are always whole runs.
 *
 * Jobs are counted, not measured, so "2.5 runs" is not a quantity — this is the
 * `allowDecimals={false}` the recharts axis carried, done once and testably.
 * Always spans 0 to a rounded top, and always has at least two entries so the
 * axis is never a single line.
 */
export const niceTicks = (peak: number, desired = 4): number[] => {
  if (!Number.isFinite(peak) || peak <= 0) return [0, 1];

  const rawStep = peak / Math.max(1, desired);
  const magnitude = 10 ** Math.floor(Math.log10(rawStep));
  const step = [1, 2, 5, 10]
    .map(multiple => multiple * magnitude)
    .find(candidate => candidate >= rawStep && candidate >= 1) ?? Math.ceil(rawStep);

  const top = Math.ceil(peak / step) * step;
  const ticks: number[] = [];
  for (let value = 0; value <= top + step / 2; value += step) ticks.push(Math.round(value));
  return ticks;
};

/** Maps bucket index and run count onto the drawing box. */
export const projectPoints = (
  values: number[],
  max: number,
  geometry: ChartGeometry,
): Point[] => {
  const { width, height, padding } = geometry;
  const innerWidth = width - padding.left - padding.right;
  const innerHeight = height - padding.top - padding.bottom;
  const lastIndex = Math.max(1, values.length - 1);
  const safeMax = max > 0 ? max : 1;

  return values.map((value, index) => ({
    x: padding.left + (innerWidth * index) / lastIndex,
    // SVG y grows downward, so a bigger count sits closer to `padding.top`.
    y: padding.top + innerHeight * (1 - value / safeMax),
  }));
};

/**
 * Tangents for a monotone cubic (Fritsch–Carlson).
 *
 * This is what `type='monotone'` bought from recharts, and it is worth keeping:
 * a plain cubic spline through run counts **overshoots**, so a day with two runs
 * sitting between two days with none dips the curve below zero and draws
 * negative jobs. Fritsch–Carlson clamps the tangents so the curve never leaves
 * the interval its endpoints define.
 */
const monotoneTangents = (points: Point[]): number[] => {
  const count = points.length;
  if (count < 2) return new Array<number>(count).fill(0);

  const h: number[] = [];
  const slope: number[] = [];
  for (let i = 0; i < count - 1; i += 1) {
    const dx = points[i + 1].x - points[i].x;
    h.push(dx);
    slope.push(dx === 0 ? 0 : (points[i + 1].y - points[i].y) / dx);
  }

  const tangents = new Array<number>(count).fill(0);
  tangents[0] = slope[0];
  tangents[count - 1] = slope[count - 2];

  for (let i = 1; i < count - 1; i += 1) {
    if (slope[i - 1] * slope[i] <= 0) {
      // A local extremum: a flat tangent is what keeps the curve from
      // overshooting past the peak it is turning at.
      tangents[i] = 0;
      continue;
    }
    const weight = h[i - 1] + h[i];
    tangents[i] = (3 * weight) / ((weight + h[i]) / slope[i - 1] + (weight + h[i - 1]) / slope[i]);
  }

  // Final clamp: keep each segment inside the Fritsch–Carlson monotonicity
  // region, which is what actually forbids the overshoot.
  for (let i = 0; i < count - 1; i += 1) {
    if (slope[i] === 0) {
      tangents[i] = 0;
      tangents[i + 1] = 0;
      continue;
    }
    const alpha = tangents[i] / slope[i];
    const beta = tangents[i + 1] / slope[i];
    const magnitude = alpha * alpha + beta * beta;
    if (magnitude > 9) {
      const scale = 3 / Math.sqrt(magnitude);
      tangents[i] = scale * alpha * slope[i];
      tangents[i + 1] = scale * beta * slope[i];
    }
  }

  return tangents.map(value => (Number.isFinite(value) ? value : 0));
};

const round = (value: number): number => Math.round(value * 100) / 100;

/** The `d` of the curve itself. Empty for no points — never a malformed path. */
export const monotoneLinePath = (points: Point[]): string => {
  if (points.length === 0) return '';
  if (points.length === 1) return `M${round(points[0].x)},${round(points[0].y)}`;

  const tangents = monotoneTangents(points);
  let path = `M${round(points[0].x)},${round(points[0].y)}`;

  for (let i = 0; i < points.length - 1; i += 1) {
    const dx = (points[i + 1].x - points[i].x) / 3;
    const c1x = points[i].x + dx;
    const c1y = points[i].y + tangents[i] * dx;
    const c2x = points[i + 1].x - dx;
    const c2y = points[i + 1].y - tangents[i + 1] * dx;
    path += `C${round(c1x)},${round(c1y)} ${round(c2x)},${round(c2y)} ${round(points[i + 1].x)},${round(points[i + 1].y)}`;
  }

  return path;
};

/** The same curve, closed down to the baseline, for the tinted fill. */
export const monotoneAreaPath = (points: Point[], baselineY: number): string => {
  const line = monotoneLinePath(points);
  if (line === '') return '';

  const first = points[0];
  const last = points[points.length - 1];
  return `${line}L${round(last.x)},${round(baselineY)}L${round(first.x)},${round(baselineY)}Z`;
};

/**
 * Which x labels to draw.
 *
 * Thirty day-numbers in the width of a card overlap into a grey smear, so this
 * keeps the ends — which is the window the reader is actually being told about —
 * and thins the middle. `interval='preserveStartEnd'`, made explicit.
 */
export const labelVisibility = (count: number, maxLabels = 8): boolean[] => {
  if (count <= maxLabels) return new Array<boolean>(count).fill(true);

  const stride = Math.ceil(count / maxLabels);
  return Array.from(
    { length: count },
    (_, index) => index === 0 || index === count - 1 || index % stride === 0,
  );
};
