//TODO: make this in function of the choosen language
const dateFormatter = new Intl.DateTimeFormat(undefined, {
  year: 'numeric',
  month: 'long',
  day: 'numeric',
  hour: 'numeric',
  minute: 'numeric',
  second: 'numeric',
});

/**
 * Calculate duration between two dates in seconds
 * @param createdAt - Start date string (ISO format)
 * @param updatedAt - End date string (ISO format)
 * @returns Duration in seconds
 */
export const calculateDuration = (createdAt: string, updatedAt: string): number => {
  const start = new Date(createdAt).getTime();
  const end = new Date(updatedAt).getTime();
  return Math.floor((end - start) / 1000);
};

/**
 * Execution duration of a job/node in seconds: from when it actually started
 * (a worker picked it up) to when it finished — NOT including the pending/queue
 * wait. Returns null while still pending (not started); counts up to now while
 * running.
 */
export const calculateExecutionDuration = (
  startedAt?: string,
  finishedAt?: string,
): number | null => {
  if (!startedAt) return null;
  const start = new Date(startedAt).getTime();
  const end = finishedAt ? new Date(finishedAt).getTime() : Date.now();
  return Math.max(0, Math.floor((end - start) / 1000));
};

/**
 * Format duration in seconds to human-readable string
 * @param seconds - Duration in seconds
 * @returns Formatted string (e.g., "1m 12s", "45s", "2h 15m")
 */
export const formatDuration = (seconds: number): string => {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) return `${hours}h ${minutes}m`;
  if (minutes > 0) return `${minutes}m ${secs}s`;
  return `${secs}s`;
};

/**
 * Format a millisecond duration. Stays in ms below a second so a fast step
 * reads as "420ms" instead of collapsing to "0s"; delegates to
 * {@link formatDuration} above that.
 * @param ms - Duration in milliseconds
 * @returns Formatted string (e.g., "420ms", "45s", "2h 15m")
 */
export const formatDurationMs = (ms: number): string =>
  ms < 1000 ? `${Math.round(ms)}ms` : formatDuration(Math.round(ms / 1000));

/**
 * Get relative time from a date string
 * @param dateString - ISO date string
 * @returns Relative time (e.g., "2m ago", "3h ago", "5d ago")
 */
export const getRelativeTime = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffDays > 0) return `${diffDays}d ago`;
  if (diffHours > 0) return `${diffHours}h ago`;
  if (diffMinutes > 0) return `${diffMinutes}m ago`;
  if (diffSeconds > 0) return `${diffSeconds}s ago`;
  return 'just now';
};

export function formatDate(isoString: string | undefined): string {
  if (!isoString) return '-';

  const date = new Date(isoString);
  if (isNaN(date.getTime())) return '-';

  return dateFormatter.format(date);
}
