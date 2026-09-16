import type { ReactNode } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { Radio, Terminal, X } from 'lucide-react';
import { Permission, useCan } from '@platform/authz';
import { cn } from '@shared/presentation/utils';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { calculateExecutionDuration, formatDuration } from '@shared/utils/date-utils.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { JobLogDisplay } from '@/modules/features/jobs/presentation/ui/jobs-log/JobLogDisplay.tsx';

interface JobNodeLogsProps {
  job: JobEntity;
  /** From the URL, in execution order. Ids no node matches are already dropped. */
  openNodeIds: readonly string[];
  isWholeJobOpen: boolean;
  /** Opens or closes one panel. No id means the whole job. */
  onTogglePanel: (nodeId?: string) => void;
}

interface LogPanelProps {
  label: string;
  closeLabel: string;
  onClose: () => void;
  children: ReactNode;
}

const LogPanel = ({ label, closeLabel, onClose, children }: LogPanelProps) => (
  <section
    aria-label={label}
    className='flex min-w-0 flex-col overflow-hidden rounded-xl border border-border shadow-sm'
  >
    <header className='flex items-center gap-2 border-b border-border bg-muted/40 px-3 py-1.5'>
      <span className='min-w-0 flex-1 truncate font-mono text-xs text-foreground'>{label}</span>
      <button
        type='button'
        onClick={onClose}
        aria-label={closeLabel}
        className='rounded-md p-1 text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground'
      >
        <X className='size-3.5' />
      </button>
    </header>
    {children}
  </section>
);

/**
 * The job's logs, as a set of panels the reader opens and closes independently:
 * comparing what two nodes printed is the point, so the nav toggles panels
 * rather than switching between them, and every open panel keeps its own live
 * stream. Closing one unmounts it, which is what cancels that stream.
 */
export const JobNodeLogs = ({
  job,
  openNodeIds,
  isWholeJobOpen,
  onTogglePanel,
}: JobNodeLogsProps) => {
  const { t } = useLingui();
  const canViewLogs = useCan(Permission.READ_JOB_LOGS);

  if (!canViewLogs) {
    return (
      <p className='text-sm italic text-muted-foreground'>
        <Trans>You don't have permission to view this job's logs</Trans>
      </p>
    );
  }

  const nodes = job.nodeExecutions.map((node, index) => ({ node, id: node.id || String(index) }));
  const openNodes = nodes.filter(({ id }) => openNodeIds.includes(id));
  const wholeJobLabel = t`Whole job`;
  const closeLabelFor = (label: string) => t`Close the logs for ${label}`;

  const buttonClassName = (isOpen: boolean) =>
    cn(
      'shrink-0 rounded-lg border border-border px-3 py-2 text-left text-sm transition-colors hover:bg-accent hover:text-accent-foreground',
      isOpen && 'border-primary bg-primary/10 text-primary',
    );

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

      <div className='flex min-h-0 flex-col gap-3 lg:flex-row'>
        <nav
          aria-label={t`Node executions`}
          className='flex gap-1.5 overflow-x-auto lg:w-60 lg:shrink-0 lg:flex-col lg:overflow-x-visible lg:overflow-y-auto'
        >
          <button
            type='button'
            onClick={() => onTogglePanel()}
            aria-pressed={isWholeJobOpen}
            className={buttonClassName(isWholeJobOpen)}
          >
            <Trans>Whole job</Trans>
          </button>

          <span aria-hidden className='w-px shrink-0 self-stretch bg-border lg:h-px lg:w-auto' />

          {nodes.map(({ node, id }) => {
            const config = getStatusConfig(node.state);
            const duration = calculateExecutionDuration(node.startedAt, node.finishedAt);
            const isOpen = openNodeIds.includes(id);

            return (
              <button
                key={id}
                type='button'
                onClick={() => onTogglePanel(id)}
                aria-pressed={isOpen}
                className={cn('flex items-center gap-2', buttonClassName(isOpen))}
              >
                <span className={cn('size-2 shrink-0 rounded-full', config.dotClassName)} />
                <span className='min-w-0 flex-1 truncate font-mono text-xs'>{id}</span>
                <span className='shrink-0 text-xs text-muted-foreground'>
                  {duration === null ? '-' : formatDuration(duration)}
                </span>
              </button>
            );
          })}
        </nav>

        <div className='flex min-h-0 min-w-0 flex-1 flex-col gap-3'>
          {isWholeJobOpen && (
            <LogPanel
              label={wholeJobLabel}
              closeLabel={closeLabelFor(wholeJobLabel)}
              onClose={() => onTogglePanel()}
            >
              <JobLogDisplay jobId={job.id} />
            </LogPanel>
          )}

          {openNodes.map(({ id }) => (
            <LogPanel
              key={id}
              label={id}
              closeLabel={closeLabelFor(id)}
              onClose={() => onTogglePanel(id)}
            >
              <JobLogDisplay jobId={job.id} nodeId={id} />
            </LogPanel>
          ))}

          {!isWholeJobOpen && openNodes.length === 0 && (
            <p className='rounded-xl border border-dashed border-border p-6 text-center text-sm text-muted-foreground'>
              <Trans>No logs open — pick the whole job or a node to read its output</Trans>
            </p>
          )}
        </div>
      </div>
    </div>
  );
};
