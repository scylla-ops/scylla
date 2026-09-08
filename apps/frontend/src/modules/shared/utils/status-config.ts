import {
  Ban,
  CheckCircle2,
  CircleHelp,
  DiamondMinusIcon,
  Loader2,
  SkipForward,
  Unplug,
  XCircle,
} from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
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

export interface StatusConfig {
  /** Lazy message: this table is built at import time, outside any i18n context. */
  label: MessageDescriptor;
  variant: 'default' | 'secondary' | 'destructive' | 'outline';
  icon: LucideIcon;
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
    variant: 'default',
    icon: Loader2,
    iconClassName: 'text-status-running animate-spin',
    barClassName: 'bg-status-running/80 animate-[smooth-pulse_2s_infinite]',
    barHoverClassName: 'ring-4 ring-status-running/30 ring-inset hover:scale-y-110',
    dotClassName: 'bg-status-running animate-pulse',
    textClassName: 'text-status-running',
  },

  pending: {
    label: msg`Pending`,
    variant: 'secondary',
    icon: DiamondMinusIcon,
    iconClassName: 'text-status-queued',
    barClassName: 'bg-status-queued/40',
    barHoverClassName: 'hover:bg-status-queued/70 hover:scale-y-110',
    dotClassName: 'bg-status-queued',
    textClassName: 'text-status-queued',
  },
  completed: {
    label: msg`Success`,
    variant: 'default',
    icon: CheckCircle2,
    iconClassName: 'text-status-passed',
    barClassName: 'bg-status-passed',
    barHoverClassName: 'hover:bg-status-passed/80 hover:scale-y-110',
    dotClassName: 'bg-status-passed',
    textClassName: 'text-status-passed',
  },
  failed: {
    label: msg`Failed`,
    variant: 'destructive',
    icon: XCircle,
    iconClassName: 'text-status-failed',
    barClassName: 'bg-status-failed/80',
    barHoverClassName: 'hover:bg-status-failed hover:scale-y-110',
    dotClassName: 'bg-status-failed',
    textClassName: 'text-status-failed',
  },
  skipped: {
    label: msg`Skipped`,
    variant: 'outline',
    icon: SkipForward,
    iconClassName: 'text-status-skipped',
    barClassName: 'bg-status-skipped/35',
    barHoverClassName: 'hover:bg-status-skipped/60 hover:scale-y-110',
    dotClassName: 'bg-status-skipped',
    textClassName: 'text-status-skipped',
  },
  orphaned: {
    label: msg`Orphaned`,
    variant: 'destructive',
    icon: Unplug,
    iconClassName: 'text-status-canceled',
    barClassName: 'bg-status-canceled/80',
    barHoverClassName: 'hover:bg-status-canceled hover:scale-y-110',
    dotClassName: 'bg-status-canceled',
    textClassName: 'text-status-canceled',
  },
  cancelled: {
    label: msg`Cancelled`,
    variant: 'outline',
    icon: Ban,
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
    variant: 'outline',
    icon: CircleHelp,
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
