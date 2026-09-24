import type { StatusState } from '@shared/presentation/ui/data-display/status-indicator.ts';

const JOB_STATUS_MAP: Record<string, StatusState> = {
  pending: 'idle',
  running: 'running',
  completed: 'success',
  success: 'success',
  failed: 'failed',
  skipped: 'skipped',
  cancelled: 'cancelled',
  orphaned: 'orphaned',
};

/** Falls back to `idle`. */
export const toStatusState = (status?: string): StatusState =>
  status ? JOB_STATUS_MAP[status.toLowerCase()] || 'idle' : 'idle';
