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

export const STATUS_CONFIG: Record<StatusKey, StatusConfig> = {
  running: {
    label: msg`Running`,
    variant: 'default',
    icon: Loader2,
    iconClassName: 'text-blue-500 animate-spin',
    barClassName: 'bg-blue-500 animate-[smooth-pulse_2s_infinite]',
    barHoverClassName: 'ring-4 ring-blue-400/30 ring-inset hover:scale-y-110',
    dotClassName: 'bg-blue-500 animate-pulse',
    textClassName: 'text-blue-600',
  },

  pending: {
    label: msg`Pending`,
    variant: 'secondary',
    icon: DiamondMinusIcon,
    iconClassName: 'text-gray-400',
    barClassName: 'bg-gray-400/80',
    barHoverClassName: 'hover:bg-gray-500 hover:scale-y-110',
    dotClassName: 'bg-gray-500',
    textClassName: 'text-gray-600',
  },
  completed: {
    label: msg`Success`,
    variant: 'default',
    icon: CheckCircle2,
    iconClassName: 'text-primary',
    barClassName: 'bg-emerald-400/80',
    barHoverClassName: 'hover:bg-emerald-500 hover:scale-y-110',
    dotClassName: 'bg-emerald-500',
    textClassName: 'text-emerald-600',
  },
  failed: {
    label: msg`Failed`,
    variant: 'destructive',
    icon: XCircle,
    iconClassName: 'text-destructive',
    barClassName: 'bg-red-400/80',
    barHoverClassName: 'hover:bg-red-500 hover:scale-y-110',
    dotClassName: 'bg-red-500',
    textClassName: 'text-red-600',
  },
  skipped: {
    label: msg`Skipped`,
    variant: 'outline',
    icon: SkipForward,
    iconClassName: 'text-zinc-500',
    barClassName: 'bg-zinc-400/70',
    barHoverClassName: 'hover:bg-zinc-500 hover:scale-y-110',
    dotClassName: 'bg-zinc-500',
    textClassName: 'text-zinc-600',
  },
  orphaned: {
    label: msg`Orphaned`,
    variant: 'destructive',
    icon: Unplug,
    iconClassName: 'text-orange-500',
    barClassName: 'bg-orange-400/80',
    barHoverClassName: 'hover:bg-orange-500 hover:scale-y-110',
    dotClassName: 'bg-orange-500',
    textClassName: 'text-orange-600',
  },
  // The server reported a state this build doesn't know about (a newer oneof
  // arm or enum value). Shown as-is rather than guessed at.
  unknown: {
    label: msg`Unknown`,
    variant: 'outline',
    icon: CircleHelp,
    iconClassName: 'text-slate-400',
    barClassName: 'bg-slate-300/80',
    barHoverClassName: 'hover:bg-slate-400 hover:scale-y-110',
    dotClassName: 'bg-slate-400',
    textClassName: 'text-slate-500',
  },
  cancelled: {
    label: msg`Cancelled`,
    variant: 'outline',
    icon: Ban,
    iconClassName: 'text-amber-500',
    barClassName: 'bg-amber-400/80',
    barHoverClassName: 'hover:bg-amber-500 hover:scale-y-110',
    dotClassName: 'bg-amber-500',
    textClassName: 'text-amber-600',
  },
};

/**
 * Resolve a status string to its config, falling back to 'pending'.
 */
export const getStatusConfig = (status: string): StatusConfig =>
  STATUS_CONFIG[status as StatusKey] ?? STATUS_CONFIG.pending;
