import { i18n } from '@lingui/core';

/**
 * Intl formatters are expensive to build, so they are cached — but keyed by
 * locale, never pinned at import time: the app language switches at runtime
 * and a module-level formatter would keep rendering dates in the language the
 * user just left. Read `i18n.locale` per call, cache the result per locale.
 */
const dateFormatters = new Map<string, Intl.DateTimeFormat>();
const relativeFormatters = new Map<string, Intl.RelativeTimeFormat>();

const currentLocale = (): string => i18n.locale || 'en';

const getDateFormatter = (key: string, options: Intl.DateTimeFormatOptions) => {
  const cacheKey = `${currentLocale()}:${key}`;
  let formatter = dateFormatters.get(cacheKey);
  if (!formatter) {
    formatter = new Intl.DateTimeFormat(currentLocale(), options);
    dateFormatters.set(cacheKey, formatter);
  }
  return formatter;
};

const getRelativeFormatter = () => {
  const locale = currentLocale();
  let formatter = relativeFormatters.get(locale);
  if (!formatter) {
    // 'short', not 'narrow': narrow renders French as a bare "-3 j" instead of
    // "il y a 3 j", which reads as a negative number rather than a past time.
    formatter = new Intl.RelativeTimeFormat(locale, { numeric: 'auto', style: 'short' });
    relativeFormatters.set(locale, formatter);
  }
  return formatter;
};

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
 * Get relative time from a date string, in the app's current language.
 * @param dateString - ISO date string
 * @returns Relative time (e.g., "2d ago" / "il y a 2 j")
 */
export const getRelativeTime = (dateString: string): string => {
  const date = new Date(dateString);
  if (isNaN(date.getTime())) return '-';

  const diffSeconds = Math.floor((Date.now() - date.getTime()) / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);
  const rtf = getRelativeFormatter();

  if (diffDays > 0) return rtf.format(-diffDays, 'day');
  if (diffHours > 0) return rtf.format(-diffHours, 'hour');
  if (diffMinutes > 0) return rtf.format(-diffMinutes, 'minute');
  if (diffSeconds > 0) return rtf.format(-diffSeconds, 'second');
  return rtf.format(0, 'second');
};

/** Full date + time, localized. */
export function formatDate(isoString: string | undefined): string {
  if (!isoString) return '-';

  const date = new Date(isoString);
  if (isNaN(date.getTime())) return '-';

  return getDateFormatter('dateTime', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
    second: 'numeric',
  }).format(date);
}

/** Calendar day only, localized — for places where the time of day is noise. */
export function formatDay(isoString: string | undefined): string {
  if (!isoString) return '-';

  const date = new Date(isoString);
  if (isNaN(date.getTime())) return '-';

  return getDateFormatter('day', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }).format(date);
}

/** Time of day only, localized. */
export function formatTime(isoString: string | undefined): string {
  if (!isoString) return '-';

  const date = new Date(isoString);
  if (isNaN(date.getTime())) return '-';

  return getDateFormatter('time', {
    hour: 'numeric',
    minute: 'numeric',
    second: 'numeric',
  }).format(date);
}
