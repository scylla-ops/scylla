/**
 * Mock runtime data for the Agent detail page.
 *
 * The control plane does not yet expose live-jobs, per-day outcomes, or an
 * agent log stream. Until those endpoints exist, this module fabricates
 * deterministic data keyed off the agent id so a given agent always renders
 * the same shape. Everything here is throwaway — replace with real fetches
 * once the backend lands.
 */

export type LogLevel = 'info' | 'ok' | 'warn' | 'error';

export interface RunningJob {
  id: string;
  pipeline: string;
  step: string;
  elapsed: string;
}

export interface PendingJob {
  id: string;
  pipeline: string;
  waiting: string;
}

export interface AgentLiveJobs {
  running: RunningJob[];
  pending: PendingJob[];
}

export interface OutcomeBucket {
  day: string;
  completed: number;
  failed: number;
  cancelled: number;
}

export interface LogLine {
  t: string;
  level: LogLevel;
  msg: string;
}

export type OutcomeRange = '24h' | '7d' | '14d' | '30d';

export const OUTCOME_RANGES: OutcomeRange[] = ['24h', '7d', '14d', '30d'];

const RANGE_DAYS: Record<OutcomeRange, number> = { '24h': 1, '7d': 7, '14d': 14, '30d': 30 };

/** Tiny deterministic string hash → 32-bit seed. */
const seedFrom = (s: string): number => {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
};

/** mulberry32 PRNG — deterministic given a seed. */
const rng = (seed: number) => {
  let a = seed;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
};

const PIPELINES = [
  'build-api',
  'web-deploy',
  'nightly-e2e',
  'lint-and-test',
  'docker-publish',
  'migrate-db',
];
const STEPS = ['checkout', 'install deps', 'compile', 'run tests', 'package', 'upload artifact'];

/** Crude ULID-ish id for mock job rows. */
const mockId = (next: () => number): string => {
  const alphabet = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
  let out = '01';
  for (let i = 0; i < 24; i++) out += alphabet[Math.floor(next() * alphabet.length)];
  return out;
};

const pick = <T>(next: () => number, arr: T[]): T => arr[Math.floor(next() * arr.length)];

/**
 * Live jobs (running + pending). `runningHint` lets the page bias the counts
 * toward the agent's real connection/stats so the mock stays plausible.
 */
export const mockLiveJobs = (agentId: string, runningHint = -1, online = true): AgentLiveJobs => {
  const next = rng(seedFrom(agentId + ':live'));
  if (!online) return { running: [], pending: [] };

  const runningCount = runningHint >= 0 ? runningHint : Math.floor(next() * 3);
  const pendingCount = Math.floor(next() * 4);

  const running: RunningJob[] = Array.from({ length: runningCount }, () => ({
    id: mockId(next),
    pipeline: pick(next, PIPELINES),
    step: pick(next, STEPS),
    elapsed: `${Math.floor(next() * 9) + 1}m ${Math.floor(next() * 59)}s`,
  }));

  const pending: PendingJob[] = Array.from({ length: pendingCount }, () => ({
    id: mockId(next),
    pipeline: pick(next, PIPELINES),
    waiting: `${Math.floor(next() * 50) + 5}s`,
  }));

  return { running, pending };
};

/** Per-day stacked-bar history for the requested range. */
export const mockOutcomes = (agentId: string, range: OutcomeRange): OutcomeBucket[] => {
  const days = RANGE_DAYS[range];
  const next = rng(seedFrom(agentId + ':outcomes:' + range));
  const today = new Date();

  return Array.from({ length: days }, (_, i) => {
    const d = new Date(today);
    d.setDate(today.getDate() - (days - 1 - i));
    const completed = Math.floor(next() * 40);
    const failed = Math.floor(next() * 6);
    const cancelled = Math.floor(next() * 3);
    return {
      day: d.toISOString().slice(0, 10),
      completed,
      failed,
      cancelled,
    };
  });
};

/** Compact per-card stats for the Agents list strip. */
export const mockCardStats = (
  agentId: string,
  online = true,
): { completed: number; running: number; hasRuns: boolean } => {
  const next = rng(seedFrom(agentId + ':card'));
  const hasRuns = next() > 0.15;
  const completed = hasRuns ? Math.floor(next() * 900) + 1 : 0;
  const running = online && hasRuns ? Math.floor(next() * 3) : 0;
  return { completed, running, hasRuns };
};

/** 12 buckets of "runs in the last hour" for the sparkline. */
export const mockSparkline = (agentId: string, online = true): number[] => {
  if (!online) return Array.from({ length: 12 }, () => 0);
  const next = rng(seedFrom(agentId + ':spark'));
  return Array.from({ length: 12 }, () => Math.floor(next() * 10));
};

const LOG_TEMPLATES: Array<{ level: LogLevel; msg: (next: () => number) => string }> = [
  { level: 'info', msg: () => 'polling control plane for work' },
  { level: 'info', msg: next => `claimed job ${mockId(next).slice(0, 10)}` },
  { level: 'info', msg: next => `running step "${pick(next, STEPS)}"` },
  {
    level: 'ok',
    msg: next => `step "${pick(next, STEPS)}" finished in ${Math.floor(next() * 30) + 1}s`,
  },
  { level: 'ok', msg: next => `job ${mockId(next).slice(0, 10)} completed` },
  { level: 'warn', msg: () => 'slow artifact upload, retrying (1/3)' },
  { level: 'warn', msg: next => `step "${pick(next, STEPS)}" took longer than expected` },
  { level: 'error', msg: next => `job ${mockId(next).slice(0, 10)} failed: exit code 1` },
  { level: 'info', msg: () => 'heartbeat sent' },
];

const fmtTime = (d: Date): string =>
  [d.getHours(), d.getMinutes(), d.getSeconds()].map(n => String(n).padStart(2, '0')).join(':');

/** Seed an initial buffer of log lines ending "just now". */
export const mockInitialLogs = (agentId: string, count = 14): LogLine[] => {
  const next = rng(seedFrom(agentId + ':logs'));
  const now = Date.now();
  return Array.from({ length: count }, (_, i) => {
    const tpl = pick(next, LOG_TEMPLATES);
    const t = new Date(now - (count - i) * (Math.floor(next() * 4000) + 1000));
    return { t: fmtTime(t), level: tpl.level, msg: tpl.msg(next) };
  });
};
