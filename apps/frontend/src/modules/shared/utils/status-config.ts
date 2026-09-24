import { msg } from '@lingui/core/macro';
import type { MessageDescriptor } from '@lingui/core';

export type StatusKey =
  | 'pending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'skipped'
  | 'cancelled'
  | 'orphaned'
  | 'unknown';

/**
 * Everything a status looks like **except its icon**.
 *
 * The icon is a component, and a file under `utils/` imports no UI library, so
 * the icons are in `presentation/ui/data-display/status-icons.ts`, keyed by
 * {@link StatusKey}.
 * The colours and labels, which are the part worth keeping in one place, stayed.
 */
export interface StatusConfig {
  /** Lazy message: this table is built at import time, outside any i18n context. */
  label: MessageDescriptor;
  /**
   * Classes for a `<Badge variant='outline'>`. The four shadcn variants can't tell
   * six statuses apart — running and completed both landed on `default` (primary),
   * so a running job read as a passed one. Tinted status tokens instead.
   */
  badgeClassName: string;
  iconClassName: string;
  barClassName: string;
  barHoverClassName: string;
  dotClassName: string;
  textClassName: string;
}

/**
 * Every colour here maps to dedicated semantic status tokens:
 *
 *   pending (queued)     → `status-queued`
 *   running              → `status-running`
 *   completed (passed)   → `status-passed`
 *   failed               → `status-failed`
 *   skipped              → `status-skipped`
 *   cancelled / orphaned → `status-canceled`
 *   unknown              → `muted-foreground`
 */
export const STATUS_CONFIG: Record<StatusKey, StatusConfig> = {
  running: {
    label: msg`Running`,
    badgeClassName: 'bg-status-running/15 border-status-running/30 text-status-running',
    iconClassName: 'text-status-running animate-spin',
    barClassName: 'bg-status-running/80 animate-[smooth-pulse_2s_infinite]',
    barHoverClassName: 'ring-4 ring-status-running/30 ring-inset hover:scale-y-110',
    dotClassName: 'bg-status-running animate-pulse',
    textClassName: 'text-status-running',
  },

  pending: {
    label: msg`Pending`,
    badgeClassName: 'bg-status-queued/15 border-status-queued/30 text-status-queued',
    iconClassName: 'text-status-queued',
    barClassName: 'bg-status-queued/40',
    barHoverClassName: 'hover:bg-status-queued/70 hover:scale-y-110',
    dotClassName: 'bg-status-queued',
    textClassName: 'text-status-queued',
  },
  completed: {
    label: msg`Success`,
    badgeClassName: 'bg-status-passed/15 border-status-passed/30 text-status-passed',
    iconClassName: 'text-status-passed',
    barClassName: 'bg-status-passed',
    barHoverClassName: 'hover:bg-status-passed/80 hover:scale-y-110',
    dotClassName: 'bg-status-passed',
    textClassName: 'text-status-passed',
  },
  failed: {
    label: msg`Failed`,
    badgeClassName: 'bg-status-failed/15 border-status-failed/30 text-status-failed',
    iconClassName: 'text-status-failed',
    barClassName: 'bg-status-failed/80',
    barHoverClassName: 'hover:bg-status-failed hover:scale-y-110',
    dotClassName: 'bg-status-failed',
    textClassName: 'text-status-failed',
  },
  skipped: {
    label: msg`Skipped`,
    badgeClassName: 'bg-status-skipped/15 border-status-skipped/30 text-status-skipped',
    iconClassName: 'text-status-skipped',
    barClassName: 'bg-status-skipped/35',
    barHoverClassName: 'hover:bg-status-skipped/60 hover:scale-y-110',
    dotClassName: 'bg-status-skipped',
    textClassName: 'text-status-skipped',
  },
  orphaned: {
    label: msg`Orphaned`,
    badgeClassName: 'bg-status-canceled/15 border-status-canceled/30 text-status-canceled',
    iconClassName: 'text-status-canceled',
    barClassName: 'bg-status-canceled/80',
    barHoverClassName: 'hover:bg-status-canceled hover:scale-y-110',
    dotClassName: 'bg-status-canceled',
    textClassName: 'text-status-canceled',
  },
  cancelled: {
    label: msg`Cancelled`,
    badgeClassName: 'bg-status-canceled/15 border-status-canceled/30 text-status-canceled',
    iconClassName: 'text-status-canceled',
    barClassName: 'bg-status-canceled/60',
    barHoverClassName: 'hover:bg-status-canceled/90 hover:scale-y-110',
    dotClassName: 'bg-status-canceled',
    textClassName: 'text-status-canceled',
  },
  // The server reported a state this build doesn't know about (a newer oneof
  // arm or enum value). Shown as-is rather than guessed at.
  unknown: {
    label: msg`Unknown`,
    badgeClassName: 'bg-muted-foreground/15 border-muted-foreground/30 text-muted-foreground',
    iconClassName: 'text-muted-foreground/60',
    barClassName: 'bg-muted-foreground/15',
    barHoverClassName: 'hover:bg-muted-foreground/40 hover:scale-y-110',
    dotClassName: 'bg-muted-foreground/60',
    textClassName: 'text-muted-foreground',
  },
};

/**
 * Resolve a status string to its config, falling back to 'pending'.
 */
export const getStatusConfig = (status: string): StatusConfig =>
  STATUS_CONFIG[status as StatusKey] ?? STATUS_CONFIG.pending;
