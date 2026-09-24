export type StatusState =
  | 'success'
  | 'failed'
  | 'running'
  | 'pending'
  | 'idle'
  | 'skipped'
  | 'cancelled'
  | 'orphaned';

export type StatusIndicatorSize = 'sm' | 'md' | 'lg';

interface StateColors {
  dot: string;
  ping: string;
  container: string;
}

/** A `Record` so that a new state fails to compile here. */
const STATE_COLORS: Record<StatusState, StateColors> = {
  success: {
    dot: 'bg-status-passed',
    ping: 'bg-status-passed/60',
    container: 'border-status-passed/30 text-status-passed',
  },
  failed: {
    dot: 'bg-status-failed',
    ping: 'bg-status-failed/60',
    container: 'border-status-failed/30 text-status-failed',
  },
  running: {
    dot: 'bg-status-running',
    ping: 'bg-status-running/60',
    container: 'border-status-running/30 text-status-running',
  },
  pending: {
    dot: 'bg-status-queued',
    ping: 'bg-status-queued/50',
    container: 'border-status-queued/30 text-status-queued',
  },
  skipped: {
    dot: 'bg-status-skipped',
    ping: 'bg-status-skipped/50',
    container: 'border-status-skipped/30 text-status-skipped',
  },
  cancelled: {
    dot: 'bg-status-canceled',
    ping: 'bg-status-canceled/50',
    container: 'border-status-canceled/30 text-status-canceled',
  },
  orphaned: {
    dot: 'bg-status-canceled',
    ping: 'bg-status-canceled/50',
    container: 'border-status-canceled/30 text-status-canceled',
  },
  idle: {
    dot: 'bg-muted-foreground/60',
    ping: 'bg-muted-foreground/30',
    container: 'border-border text-muted-foreground',
  },
};

const SIZE_CLASSES: Record<StatusIndicatorSize, { dot: string; container: string }> = {
  sm: { dot: 'h-2 w-2', container: 'px-2 py-1 text-xs' },
  md: { dot: 'h-3 w-3', container: 'px-3 py-1.5 text-sm' },
  lg: { dot: 'h-4 w-4', container: 'px-4 py-2 text-sm' },
};

export const statusStateColors = (state: StatusState) => STATE_COLORS[state] ?? STATE_COLORS.idle;

export const statusIndicatorSize = (size: StatusIndicatorSize) => SIZE_CLASSES[size];
