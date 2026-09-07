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
 * Every colour here is a theme token, never a Tailwind palette shade: the app
 * ships a light and a dark theme, and a hard-coded `emerald-500` only ever
 * looks right in one of them. The mapping is semantic —
 *
 *   in progress (running)                 → `info` (the theme's blue)
 *   success (completed)                   → `primary`
 *   failure                               → `destructive`
 *   interrupted (cancelled / orphaned)    → `warning`
 *   inert (pending / skipped / unknown)   → `muted-foreground`
 *
 * — and states that share a token are separated by opacity, so a bar of mixed
 * statuses stays readable without inventing a colour outside the theme.
 */
export const STATUS_CONFIG: Record<StatusKey, StatusConfig> = {
  running: {
    label: msg`Running`,
    variant: 'default',
    icon: Loader2,
    iconClassName: 'text-info animate-spin',
    barClassName: 'bg-info/80 animate-[smooth-pulse_2s_infinite]',
    barHoverClassName: 'ring-4 ring-info/30 ring-inset hover:scale-y-110',
    dotClassName: 'bg-info animate-pulse',
    textClassName: 'text-info',
  },

  pending: {
    label: msg`Pending`,
    variant: 'secondary',
    icon: DiamondMinusIcon,
    iconClassName: 'text-muted-foreground',
    barClassName: 'bg-muted-foreground/40',
    barHoverClassName: 'hover:bg-muted-foreground/70 hover:scale-y-110',
    dotClassName: 'bg-muted-foreground',
    textClassName: 'text-muted-foreground',
  },
  completed: {
    label: msg`Success`,
    variant: 'default',
    icon: CheckCircle2,
    iconClassName: 'text-primary',
    barClassName: 'bg-primary',
    barHoverClassName: 'hover:bg-primary/80 hover:scale-y-110',
    dotClassName: 'bg-primary',
    textClassName: 'text-primary',
  },
  failed: {
    label: msg`Failed`,
    variant: 'destructive',
    icon: XCircle,
    iconClassName: 'text-destructive',
    barClassName: 'bg-destructive/80',
    barHoverClassName: 'hover:bg-destructive hover:scale-y-110',
    dotClassName: 'bg-destructive',
    textClassName: 'text-destructive',
  },
  skipped: {
    label: msg`Skipped`,
    variant: 'outline',
    icon: SkipForward,
    iconClassName: 'text-muted-foreground/70',
    barClassName: 'bg-muted-foreground/25',
    barHoverClassName: 'hover:bg-muted-foreground/50 hover:scale-y-110',
    dotClassName: 'bg-muted-foreground/70',
    textClassName: 'text-muted-foreground',
  },
  orphaned: {
    label: msg`Orphaned`,
    variant: 'destructive',
    icon: Unplug,
    iconClassName: 'text-warning',
    barClassName: 'bg-warning',
    barHoverClassName: 'hover:bg-warning/80 hover:scale-y-110',
    dotClassName: 'bg-warning',
    textClassName: 'text-warning',
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
  cancelled: {
    label: msg`Cancelled`,
    variant: 'outline',
    icon: Ban,
    iconClassName: 'text-warning/80',
    barClassName: 'bg-warning/60',
    barHoverClassName: 'hover:bg-warning/90 hover:scale-y-110',
    dotClassName: 'bg-warning/80',
    textClassName: 'text-warning',
  },
};

/**
 * Resolve a status string to its config, falling back to 'pending'.
 */
export const getStatusConfig = (status: string): StatusConfig =>
  STATUS_CONFIG[status as StatusKey] ?? STATUS_CONFIG.pending;
