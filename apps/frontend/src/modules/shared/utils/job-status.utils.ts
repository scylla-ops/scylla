// From the Svelte port, not the React original: its only consumer is
// `features/pipeline`, which is Svelte as of Phase 5, and `ui/` is deleted in
// Phase 6 while `ui/` is renamed into its place.
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

/**
 * Maps a raw job status string (from the API) to a StatusState used by UI components.
 * Falls back to 'idle' for unknown or undefined values.
 */
export const toStatusState = (status?: string): StatusState =>
  status ? JOB_STATUS_MAP[status.toLowerCase()] || 'idle' : 'idle';
