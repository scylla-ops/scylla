import { Trans, useLingui } from '@lingui/react/macro';
import { Radio, Terminal } from 'lucide-react';
import { Permission, useCan } from '@platform/authz';
import { cn } from '@shared/presentation/utils';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { calculateExecutionDuration, formatDuration } from '@shared/utils/date-utils.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { JobLogDisplay } from '@/modules/features/jobs/presentation/ui/jobs-log/JobLogDisplay.tsx';

interface JobNodeLogsProps {
  job: JobEntity;
  /** From the URL. An id no node matches falls back to the whole job. */
  selectedNodeId?: string;
  onSelectNode: (nodeId?: string) => void;
}

export const JobNodeLogs = ({ job, selectedNodeId, onSelectNode }: JobNodeLogsProps) => {
  const { t } = useLingui();
  const canViewLogs = useCan(Permission.READ_JOB_LOGS);

  if (!canViewLogs) {
    return (
      <p className='text-sm italic text-muted-foreground'>
        <Trans>You don't have permission to view this job's logs</Trans>
      </p>
    );
  }

  const selected = job.nodeExecutions.find(node => node.id === selectedNodeId);

  return (
    <div className='flex min-h-0 flex-col gap-3'>
      <div className='flex items-center gap-2'>
        <div className='flex size-8 items-center justify-center rounded-lg bg-primary/10'>
          <Terminal className='size-4 text-primary' />
        </div>
        <h2 className='text-lg font-semibold text-foreground'>
          <Trans>Logs</Trans>
        </h2>
        <span className='flex items-center gap-1.5 text-sm text-muted-foreground'>
          <Radio className='size-4 animate-pulse text-green-500' />
          <Trans>Streaming live output</Trans>
        </span>
      </div>

      <div className='flex flex-col gap-3 lg:flex-row'>
        <nav
          aria-label={t`Node executions`}
          className='flex gap-1.5 overflow-x-auto lg:w-60 lg:shrink-0 lg:flex-col lg:overflow-x-visible'
        >
          <button
            type='button'
            onClick={() => onSelectNode()}
            aria-current={selected ? undefined : 'true'}
            className={cn(
              'shrink-0 rounded-lg border border-border px-3 py-2 text-left text-sm transition-colors hover:bg-accent hover:text-accent-foreground',
              !selected && 'border-primary bg-primary/10 text-primary',
            )}
          >
            <Trans>Whole job</Trans>
          </button>

          {job.nodeExecutions.map((node, index) => {
            const nodeId = node.id || String(index);
            const config = getStatusConfig(node.state);
            const duration = calculateExecutionDuration(node.startedAt, node.finishedAt);
            const isSelected = selected?.id === node.id;

            return (
              <button
                key={nodeId}
                type='button'
                onClick={() => onSelectNode(nodeId)}
                aria-current={isSelected ? 'true' : undefined}
                className={cn(
                  'flex shrink-0 items-center gap-2 rounded-lg border border-border px-3 py-2 text-left text-sm transition-colors hover:bg-accent hover:text-accent-foreground',
                  isSelected && 'border-primary bg-primary/10 text-primary',
                )}
              >
                <span className={cn('size-2 shrink-0 rounded-full', config.dotClassName)} />
                <span className='min-w-0 flex-1 truncate font-mono text-xs'>{nodeId}</span>
                <span className='shrink-0 text-xs text-muted-foreground'>
                  {duration === null ? '-' : formatDuration(duration)}
                </span>
              </button>
            );
          })}
        </nav>

        {/* Keyed so switching node remounts the viewer instead of replaying one
            stream's lines into the document another stream opened. */}
        <div className='min-w-0 flex-1'>
          <JobLogDisplay key={selected?.id ?? 'job'} jobId={job.id} nodeId={selected?.id} />
        </div>
      </div>
    </div>
  );
};
