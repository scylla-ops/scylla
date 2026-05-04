import { CheckCircle2, Clock, Loader2, XCircle } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';

export type StatusKey = 'pending' | 'running' | 'completed' | 'failed';

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
  pending: {
    label: 'Pending',
    variant: 'secondary',
    icon: Clock,
    iconClassName: 'text-slate-500',
    barClassName: 'bg-slate-300',
    barHoverClassName: 'hover:bg-slate-400 hover:scale-y-110',
    dotClassName: 'bg-slate-400',
    textClassName: 'text-slate-600',
  },
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
};

/**
 * Resolve a status string to its config, falling back to 'pending'.
 */
export const getStatusConfig = (status: string): StatusConfig =>
  STATUS_CONFIG[status as StatusKey] ?? STATUS_CONFIG.pending;
