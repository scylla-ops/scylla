import type { ReactNode } from 'react';
import { Activity, CheckCircle2, XCircle, Ban, Unplug, Loader2 } from 'lucide-react';
import { Trans } from '@lingui/react/macro';
import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { Skeleton } from '@shadcn/skeleton.tsx';
import { getRelativeTime } from '@shared/utils/date-utils.ts';
import type { JobsSummary } from '@/modules/features/jobs';

interface OutcomeProps {
  icon: ReactNode;
  label: ReactNode;
  value: number;
}

const Outcome = ({ icon, label, value }: OutcomeProps) => (
  <div className='flex items-center gap-2'>
    <div className='shrink-0'>{icon}</div>
    <div className='min-w-0'>
      <p className='text-lg font-semibold leading-none tabular-nums'>{value}</p>
      <p className='truncate text-xs text-muted-foreground mt-0.5'>{label}</p>
    </div>
  </div>
);

interface RunActivityCardProps {
  summary: JobsSummary;
  /** Runs the organization has in total, which may exceed the summarized window. */
  totalRuns: number;
  /** True when `summary` covers only the most recent page of runs. */
  truncated: boolean;
  loading: boolean;
}

/**
 * Organization-wide run activity, from `ListOrganizationJobs`.
 *
 * The backend exposes no organization-level aggregate — only a paginated job
 * listing — so every figure here is computed over the most recent window. When
 * that window does not cover the whole history the card says so rather than
 * presenting a partial success rate as if it were the all-time one.
 */
export const RunActivityCard = ({
  summary,
  totalRuns,
  truncated,
  loading,
}: RunActivityCardProps) => {
  const inFlight = summary.pending + summary.running;

  return (
    <Card className='py-5'>
      <CardHeader className='px-5 pb-0'>
        <CardTitle className='flex items-center justify-between text-base font-semibold'>
          <span className='flex items-center gap-2'>
            <Activity className='h-4 w-4 text-primary' />
            <Trans>Run activity</Trans>
          </span>
          {!loading && inFlight > 0 && (
            <span className='flex items-center gap-1.5 text-xs font-normal text-muted-foreground'>
              <Loader2 className='h-3 w-3 animate-spin text-primary' />
              <Trans>{inFlight} in progress</Trans>
            </span>
          )}
        </CardTitle>
      </CardHeader>

      <CardContent className='px-5 pb-0 pt-4'>
        {loading ? (
          <div className='grid grid-cols-2 gap-4 sm:grid-cols-4'>
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={i} className='h-10 w-full' />
            ))}
          </div>
        ) : summary.total === 0 ? (
          <p className='text-sm text-muted-foreground'>
            <Trans>No pipeline has run yet.</Trans>
          </p>
        ) : (
          <>
            <div className='grid grid-cols-2 gap-4 sm:grid-cols-4'>
              <Outcome
                icon={<CheckCircle2 className='h-4 w-4 text-[var(--success)]' />}
                label={<Trans>Completed</Trans>}
                value={summary.completed}
              />
              <Outcome
                icon={<XCircle className='h-4 w-4 text-destructive' />}
                label={<Trans>Failed</Trans>}
                value={summary.failed}
              />
              <Outcome
                icon={<Ban className='h-4 w-4 text-muted-foreground' />}
                label={<Trans>Cancelled</Trans>}
                value={summary.cancelled}
              />
              <Outcome
                icon={<Unplug className='h-4 w-4 text-amber-500' />}
                label={<Trans>Orphaned</Trans>}
                value={summary.orphaned}
              />
            </div>

            <p className='mt-4 text-xs text-muted-foreground'>
              {summary.lastRunAt && (
                <>
                  <Trans>Last run {getRelativeTime(summary.lastRunAt)}</Trans>
                  {' · '}
                </>
              )}
              {truncated ? (
                <Trans>
                  over the last {summary.total} of {totalRuns} runs
                </Trans>
              ) : (
                <Trans>over all {summary.total} runs</Trans>
              )}
            </p>
          </>
        )}
      </CardContent>
    </Card>
  );
};
