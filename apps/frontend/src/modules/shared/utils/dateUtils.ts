/**
 * Calculate duration between two dates in seconds
 * @param createdAt - Start date string (ISO format)
 * @param updatedAt - End date string (ISO format)
 * @returns Duration in seconds
 */
export const calculateDuration = (createdAt: string, updatedAt: string): number => {
  const start = new Date(createdAt).getTime();
  const end = new Date(updatedAt).getTime();
  return Math.floor((end - start) / 1000); // in seconds
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

