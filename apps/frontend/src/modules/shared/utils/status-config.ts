import { Ban, CheckCircle2, DiamondMinusIcon, Loader2, SkipForward, Unplug, XCircle } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';

export type StatusKey =
  | 'pending'
  | 'running'
  | 'completed'
  | 'failed'
  | 'skipped'
  | 'cancelled'
  | 'orphaned';

export interface StatusConfig {
  label: string;
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
    label: 'Running',
    variant: 'default',
    icon: Loader2,
    iconClassName: 'text-blue-500 animate-spin',
    barClassName: 'bg-blue-500 animate-[smooth-pulse_2s_infinite]',
    barHoverClassName: 'ring-4 ring-blue-400/30 ring-inset hover:scale-y-110',
    dotClassName: 'bg-blue-500 animate-pulse',
    textClassName: 'text-blue-600',
  },

  pending: {
    label: 'Pending',
    variant: 'secondary',
    icon: DiamondMinusIcon,
    iconClassName: 'text-gray-400',
    barClassName: 'bg-gray-400/80',
    barHoverClassName: 'hover:bg-gray-500 hover:scale-y-110',
    dotClassName: 'bg-gray-500',
    textClassName: 'text-gray-600',
  },
  completed: {
    label: 'Success',
    variant: 'default',
    icon: CheckCircle2,
    iconClassName: 'text-primary',
    barClassName: 'bg-emerald-400/80',
    barHoverClassName: 'hover:bg-emerald-500 hover:scale-y-110',
    dotClassName: 'bg-emerald-500',
    textClassName: 'text-emerald-600',
  },
  failed: {
    label: 'Failed',
    variant: 'destructive',
    icon: XCircle,
    iconClassName: 'text-destructive',
    barClassName: 'bg-red-400/80',
    barHoverClassName: 'hover:bg-red-500 hover:scale-y-110',
    dotClassName: 'bg-red-500',
    textClassName: 'text-red-600',
  },
  skipped: {
    label: 'Skipped',
    variant: 'outline',
    icon: SkipForward,
    iconClassName: 'text-zinc-500',
    barClassName: 'bg-zinc-400/70',
    barHoverClassName: 'hover:bg-zinc-500 hover:scale-y-110',
    dotClassName: 'bg-zinc-500',
    textClassName: 'text-zinc-600',
  },
  orphaned: {
    label: 'Orphaned',
    variant: 'destructive',
    icon: Unplug,
    iconClassName: 'text-orange-500',
    barClassName: 'bg-orange-400/80',
    barHoverClassName: 'hover:bg-orange-500 hover:scale-y-110',
    dotClassName: 'bg-orange-500',
    textClassName: 'text-orange-600',
  },
  cancelled: {
    label: 'Cancelled',
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
