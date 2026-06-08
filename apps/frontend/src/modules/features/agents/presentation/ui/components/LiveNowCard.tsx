import { Trans } from '@lingui/react/macro';
import { ArrowUpRight } from 'lucide-react';
import { Card, CardContent } from '@shadcn';
import { cn } from '@shared/presentation/utils';
import type { AgentLiveJobs } from '@/modules/features/agents/presentation/utils/agent-mock-data.ts';

interface LiveNowCardProps {
  jobs: AgentLiveJobs;
}

const SectionHeader = ({
  tone,
  label,
  count,
}: {
  tone: 'running' | 'pending';
  label: React.ReactNode;
  count: number;
}) => {
  const active = count > 0;
  const dotColor = tone === 'running' ? 'bg-success' : 'bg-warning';
  return (
    <div className='flex items-center gap-2'>
      <span className='relative flex h-2.5 w-2.5'>
        {tone === 'running' && active && (
          <span className='absolute inline-flex h-full w-full animate-ping rounded-full bg-success/70' />
        )}
        <span
          className={cn(
            'relative inline-flex h-2.5 w-2.5 rounded-full',
            dotColor,
            !active && 'opacity-30',
          )}
        />
      </span>
      <span className='font-mono text-xs uppercase tracking-wide text-muted-foreground'>
        {label}
      </span>
      <span className='text-lg font-semibold leading-none'>{count}</span>
    </div>
  );
};

// Job rows are styled per spec but inert for now: live-jobs are mocked and the
// standalone /jobs/:id route does not exist yet. Wire navigation when both land.
const JobRow = ({
  tone,
  id,
  pipeline,
  line2,
  accentValue,
}: {
  tone: 'running' | 'pending';
  id: string;
  pipeline: string;
  line2: React.ReactNode;
  accentValue: string;
}) => {
  const accentText = tone === 'running' ? 'text-success' : 'text-warning';
  return (
    <div
      className={cn(
        'group rounded border-l-[3px] p-2 pl-2.5 transition-colors',
        tone === 'running'
          ? 'border-l-success bg-success/10 hover:bg-success/5'
          : 'border-l-warning bg-warning/10 hover:bg-warning/5',
        'hover:border hover:border-l-[3px]',
        tone === 'running' ? 'hover:border-success' : 'hover:border-warning',
      )}
    >
      <div className='flex items-center gap-1.5 text-xs'>
        <span className='font-mono font-semibold'>{id.slice(0, 12)}…</span>
        <span className='text-muted-foreground'>·</span>
        <span className={cn('truncate font-semibold', accentText)}>{pipeline}</span>
        <ArrowUpRight className='ml-auto h-3 w-3 shrink-0 opacity-40 transition-opacity group-hover:opacity-100' />
      </div>
      <div className='mt-0.5 font-mono text-[10px] text-muted-foreground'>{line2}</div>
      <span className='sr-only'>{accentValue}</span>
    </div>
  );
};

export const LiveNowCard = ({ jobs }: LiveNowCardProps) => {
  return (
    <Card>
      <CardContent className='space-y-4 p-4'>
        {/* Running */}
        <div className='space-y-2'>
          <SectionHeader
            tone='running'
            label={<Trans>running</Trans>}
            count={jobs.running.length}
          />
          {jobs.running.length > 0 ? (
            <div className='space-y-1.5'>
              {jobs.running.map(j => (
                <JobRow
                  key={j.id}
                  tone='running'
                  id={j.id}
                  pipeline={j.pipeline}
                  accentValue={j.elapsed}
                  line2={
                    <>
                      {j.step} · <span className='text-success'>{j.elapsed}</span>
                    </>
                  }
                />
              ))}
            </div>
          ) : (
            <div className='rounded border border-dashed bg-muted/40 p-2 text-center text-xs text-muted-foreground'>
              <Trans>idle — no jobs</Trans>
            </div>
          )}
        </div>

        {/* Pending */}
        <div className='space-y-2'>
          <SectionHeader
            tone='pending'
            label={<Trans>pending</Trans>}
            count={jobs.pending.length}
          />
          {jobs.pending.length > 0 ? (
            <div className='space-y-1.5'>
              {jobs.pending.map(j => (
                <JobRow
                  key={j.id}
                  tone='pending'
                  id={j.id}
                  pipeline={j.pipeline}
                  accentValue={j.waiting}
                  line2={
                    <>
                      <Trans>waiting</Trans>{' '}
                      <span className='text-warning'>{j.waiting}</span>
                    </>
                  }
                />
              ))}
            </div>
          ) : (
            <div className='rounded border border-dashed bg-muted/40 p-2 text-center text-xs text-muted-foreground'>
              <Trans>queue empty</Trans>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};
