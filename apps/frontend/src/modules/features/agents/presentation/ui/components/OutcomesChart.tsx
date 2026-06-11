import { useMemo, useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { Card, CardContent } from '@shadcn';
import { cn } from '@shared/presentation/utils';
import type { DailyOutcome } from '@/modules/features/agents/domain/models/agent.model.ts';

interface OutcomesChartProps {
  /** Per-day finished-job outcomes from the stats endpoint (gap days absent). */
  daily: DailyOutcome[];
  /** All-time aggregates for the legend (from the stats endpoint). */
  aggregate: { completed: number; failed: number; cancelled: number };
}

type OutcomeRange = '7d' | '14d' | '30d';
const OUTCOME_RANGES: OutcomeRange[] = ['7d', '14d', '30d'];
const RANGE_DAYS: Record<OutcomeRange, number> = { '7d': 7, '14d': 14, '30d': 30 };

interface Bucket {
  day: string;
  completed: number;
  failed: number;
  cancelled: number;
}

/** Local calendar date (yyyy-mm-dd) of an ISO timestamp. */
const localDay = (iso: string): string => {
  const d = new Date(iso);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(
    d.getDate(),
  ).padStart(2, '0')}`;
};

/** Zero-fill the last `days` calendar days from the sparse backend series. */
const fillBuckets = (daily: DailyOutcome[], days: number): Bucket[] => {
  const byDay = new Map(daily.map(d => [localDay(d.day), d]));
  const today = new Date();
  return Array.from({ length: days }, (_, i) => {
    const d = new Date(today);
    d.setDate(today.getDate() - (days - 1 - i));
    const key = localDay(d.toISOString());
    const hit = byDay.get(key);
    return {
      day: key,
      completed: hit?.completed ?? 0,
      failed: hit?.failed ?? 0,
      cancelled: hit?.cancelled ?? 0,
    };
  });
};

const SEGMENTS = [
  { key: 'completed', color: 'var(--success)' },
  { key: 'failed', color: 'var(--destructive)' },
  { key: 'cancelled', color: 'var(--warning)' },
] as const;

export const OutcomesChart = ({ daily, aggregate }: OutcomesChartProps) => {
  const [range, setRange] = useState<OutcomeRange>('14d');
  const [hover, setHover] = useState<number | null>(null);

  const buckets = useMemo(() => fillBuckets(daily, RANGE_DAYS[range]), [daily, range]);
  const windowTotal = buckets.reduce((n, b) => n + b.completed + b.failed + b.cancelled, 0);
  const max = Math.max(1, ...buckets.map(b => b.completed + b.failed + b.cancelled));

  const monthLabel = buckets.length
    ? new Date(buckets[0].day).toLocaleString('en-US', { month: 'short' })
    : '';

  return (
    <Card>
      <CardContent className='space-y-3 p-4'>
        <div className='flex items-center justify-between'>
          <span className='font-mono text-xs uppercase tracking-wide text-muted-foreground'>
            <Trans>outcomes · last</Trans> {range}
          </span>
          <div className='flex overflow-hidden rounded border text-[11px] font-mono'>
            {OUTCOME_RANGES.map(r => (
              <button
                key={r}
                type='button'
                onClick={() => setRange(r)}
                className={cn(
                  'px-2 py-0.5 transition-colors',
                  r === range
                    ? 'bg-foreground text-background'
                    : 'text-muted-foreground hover:bg-muted',
                )}
              >
                {r}
              </button>
            ))}
          </div>
        </div>

        {windowTotal === 0 ? (
          <div className='flex h-[140px] flex-col items-center justify-center gap-1 text-center'>
            <p className='text-sm text-muted-foreground'>
              <Trans>No finished jobs in this window.</Trans>
            </p>
            <p className='text-xs text-muted-foreground/70'>
              <Trans>Run a pipeline on this agent and the chart fills up.</Trans>
            </p>
          </div>
        ) : (
          <div className='relative pl-7'>
            {/* Y ticks */}
            <div className='pointer-events-none absolute left-0 top-0 flex h-[120px] w-6 flex-col justify-between font-mono text-[9px] text-muted-foreground'>
              <span>{max}</span>
              <span>{Math.round(max / 2)}</span>
              <span>0</span>
            </div>

            <div className='relative h-[120px] border-b border-foreground/80'>
              {/* gridlines */}
              <div className='pointer-events-none absolute inset-0 flex flex-col justify-between'>
                <div className='border-t border-dashed border-border' />
                <div className='border-t border-dashed border-border' />
                <div className='border-t border-transparent' />
              </div>

              <div className='flex h-full items-end gap-px'>
                {buckets.map((b, i) => {
                  const total = b.completed + b.failed + b.cancelled;
                  return (
                    <div
                      key={b.day}
                      // h-full is load-bearing: the bar's percentage height
                      // resolves against this wrapper — without it every bar
                      // computes to 0 and the chart renders empty.
                      className='group relative flex h-full flex-1 flex-col justify-end'
                      title={`${b.day}: ${b.completed} completed, ${b.failed} failed, ${b.cancelled} cancelled`}
                      onMouseEnter={() => setHover(i)}
                      onMouseLeave={() => setHover(h => (h === i ? null : h))}
                    >
                      <div
                        className={cn(
                          'mx-auto flex w-full max-w-[28px] flex-col-reverse transition-opacity',
                          hover === null || hover === i ? 'opacity-100' : 'opacity-60',
                        )}
                        style={{ height: `${(total / max) * 100}%` }}
                      >
                        {SEGMENTS.map(seg => {
                          const v = b[seg.key];
                          if (!v) return null;
                          return (
                            <div
                              key={seg.key}
                              style={{ height: `${(v / total) * 100}%`, background: seg.color }}
                            />
                          );
                        })}
                      </div>

                      {/* Tooltip */}
                      {hover === i && (
                        <div className='pointer-events-none absolute -top-2 left-1/2 z-10 w-max -translate-x-1/2 -translate-y-full rounded bg-foreground px-2 py-1.5 text-[10px] text-background shadow-md'>
                          <p className='font-semibold'>
                            {b.day} · {total} <Trans>runs</Trans>
                          </p>
                          {b.completed > 0 && (
                            <p>
                              <span style={{ color: 'var(--success)' }}>●</span> completed{' '}
                              {b.completed}
                            </p>
                          )}
                          {b.failed > 0 && (
                            <p>
                              <span style={{ color: 'var(--destructive)' }}>●</span> failed{' '}
                              {b.failed}
                            </p>
                          )}
                          {b.cancelled > 0 && (
                            <p>
                              <span style={{ color: 'var(--warning)' }}>●</span> cancelled{' '}
                              {b.cancelled}
                            </p>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>

            {/* X labels — thin out on the 30d window so they stay readable */}
            <div className='mt-1 flex gap-px pl-0'>
              <span className='absolute left-0 font-mono text-[9px] text-muted-foreground'>
                {monthLabel}
              </span>
              {buckets.map((b, i) => (
                <span
                  key={b.day}
                  className={cn(
                    'flex-1 text-center font-mono text-[9px]',
                    hover === i ? 'text-foreground' : 'text-muted-foreground',
                  )}
                >
                  {buckets.length > 16 && i % 2 === 1 && hover !== i
                    ? ''
                    : new Date(b.day).getDate()}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Legend */}
        <div className='flex flex-wrap items-center gap-x-4 gap-y-1 text-xs'>
          <span className='flex items-center gap-1.5'>
            <span className='h-2.5 w-2.5 rounded-sm' style={{ background: 'var(--success)' }} />
            <span className='font-semibold'>{aggregate.completed}</span> <Trans>completed</Trans>
          </span>
          <span className='flex items-center gap-1.5'>
            <span className='h-2.5 w-2.5 rounded-sm' style={{ background: 'var(--destructive)' }} />
            <span className='font-semibold'>{aggregate.failed}</span> <Trans>failed</Trans>
          </span>
          <span className='flex items-center gap-1.5'>
            <span className='h-2.5 w-2.5 rounded-sm' style={{ background: 'var(--warning)' }} />
            <span className='font-semibold'>{aggregate.cancelled}</span> <Trans>cancelled</Trans>
          </span>
          <span className='ml-auto font-mono text-[11px] text-muted-foreground'>
            {aggregate.completed + aggregate.failed + aggregate.cancelled} <Trans>finished</Trans>
          </span>
        </div>
      </CardContent>
    </Card>
  );
};
