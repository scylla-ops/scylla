import type { StatusState } from '@shared/presentation/ui/data-display/status-indicator';

const JOB_STATUS_MAP: Record<string, StatusState> = {
  pending: 'idle',
  running: 'running',
  completed: 'success',
  success: 'success',
  failed: 'failed',
  skipped: 'skipped',
  cancelled: 'cancelled',
};

/**
 * Maps a raw job status string (from the API) to a StatusState used by UI components.
 * Falls back to 'idle' for unknown or undefined values.
 */
export const toStatusState = (status?: string): StatusState =>
  status ? JOB_STATUS_MAP[status.toLowerCase()] || 'idle' : 'idle';
