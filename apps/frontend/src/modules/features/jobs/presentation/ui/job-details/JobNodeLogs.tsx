import type { ReactNode } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { Radio, Terminal, X } from 'lucide-react';
import { Permission, useCan } from '@platform/authz';
import { cn } from '@shared/presentation/utils';
import { useMeasuredHeight } from '@shared/presentation/hooks/use-measured-height.ts';
import { getStatusConfig } from '@shared/utils/status-config.ts';
import { calculateExecutionDuration, formatDuration } from '@shared/utils/date-utils.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import { JobLogDisplay } from '@/modules/features/jobs/presentation/ui/jobs-log/JobLogDisplay.tsx';

/** The fixed `h-9` header standing above each log. */
const PANEL_HEADER_HEIGHT = 36;
/** The panel's own `border` (1px top + 1px bottom), on top of the header. */
const PANEL_BORDER_HEIGHT = 2;
/** Below this a log is a peephole; the column scrolls rather than shrink past it. */
const MIN_LOG_HEIGHT = 192;
/** What a node's log stands at, however many are open — the column takes the overflow. */
const NODE_LOG_HEIGHT = 448;

/** The whole job is only ever shown alone, so its log gets the column entire. */
const wholeJobLogHeight = (columnHeight: number | null): number | undefined =>
  columnHeight === null
    ? undefined
    : Math.max(MIN_LOG_HEIGHT, columnHeight - PANEL_HEADER_HEIGHT - PANEL_BORDER_HEIGHT);

interface JobNodeLogsProps {
  job: JobEntity;
  /** From the URL, in execution order. Ids no node matches are already dropped. */
  openNodeIds: readonly string[];
  isWholeJobOpen: boolean;
  /** Adds or removes one node's panel, leaving the other open ones alone. */
  onToggleNode: (nodeId: string) => void;
  /** Drops every node panel, which is what brings the whole job back. */
  onShowWholeJob: () => void;
}

interface LogPanelProps {
  label: string;
  closeLabel?: string;
  onClose?: () => void;
  children: ReactNode;
}

const LogPanel = ({ label, closeLabel, onClose, children }: LogPanelProps) => (
  <section
    aria-label={label}
    className='flex min-w-0 shrink-0 flex-col overflow-hidden rounded-xl border border-border shadow-sm'
  >
    <header className='flex h-9 shrink-0 items-center gap-2 border-b border-border bg-muted/40 px-3'>
      <span className='min-w-0 flex-1 truncate font-mono text-xs text-foreground'>{label}</span>
      {onClose && (
        <button
          type='button'
          onClick={onClose}
          aria-label={closeLabel}
          className='rounded-md p-1 text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground'
        >
          <X className='size-3.5' />
        </button>
      )}
    </header>
    {children}
  </section>
);

/**
 * The job's logs: the job as a whole, or the nodes the reader picked out of it.
 *
 * Comparing what two nodes printed is the point, so the nav adds and removes
 * node panels rather than switching between them, and each keeps the same
 * readable height whether it is alone or one of five — past the room the page
 * has, the column scrolls. The whole job is what shows when no node is picked,
 * never a panel alongside them, which is why it has no close button: closing
 * the last node is what comes back to it.
 *
 * Every open panel keeps its own live stream, and closing one unmounts it,
 * which is what cancels that stream.
 */
export const JobNodeLogs = ({
  job,
  openNodeIds,
  isWholeJobOpen,
  onToggleNode,
  onShowWholeJob,
}: JobNodeLogsProps) => {
  const { t } = useLingui();
  const canViewLogs = useCan(Permission.READ_JOB_LOGS);
  const { height, containerRef } = useMeasuredHeight();

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
    <div className='flex min-h-0 flex-1 flex-col gap-3'>
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

      <div className='flex min-h-0 flex-1 flex-col gap-3 lg:flex-row'>
        <nav
          aria-label={t`Node executions`}
          className='flex shrink-0 gap-1.5 overflow-x-auto lg:w-60 lg:flex-col lg:overflow-x-visible lg:overflow-y-auto'
        >
          <button
            type='button'
            onClick={onShowWholeJob}
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
                onClick={() => onToggleNode(id)}
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

        {/* The column's height comes from the layout and never from the panels
            inside it, or measuring it would resize what it measures. */}
        <div
          ref={containerRef}
          className='flex min-h-0 min-w-0 flex-1 flex-col gap-3 overflow-y-auto'
        >
          {isWholeJobOpen ? (
            <LogPanel label={wholeJobLabel}>
              <JobLogDisplay jobId={job.id} maxHeight={wholeJobLogHeight(height)} />
            </LogPanel>
          ) : (
            openNodes.map(({ id }) => (
              <LogPanel
                key={id}
                label={id}
                closeLabel={closeLabelFor(id)}
                onClose={() => onToggleNode(id)}
              >
                <JobLogDisplay jobId={job.id} nodeId={id} maxHeight={NODE_LOG_HEIGHT} />
              </LogPanel>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
