import BanIcon from '@lucide/svelte/icons/ban';
import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
import CircleHelpIcon from '@lucide/svelte/icons/circle-help';
import DiamondMinusIcon from '@lucide/svelte/icons/diamond-minus';
import Loader2Icon from '@lucide/svelte/icons/loader-2';
import SkipForwardIcon from '@lucide/svelte/icons/skip-forward';
import UnplugIcon from '@lucide/svelte/icons/unplug';
import XCircleIcon from '@lucide/svelte/icons/x-circle';
import type { StatusKey } from '@shared/utils/status-config.ts';
import type { LucideIcon } from '../icon.ts';

/** A `Record` so that a new status fails to compile here instead of rendering nothing. */
export const STATUS_ICONS: Record<StatusKey, LucideIcon> = {
  running: Loader2Icon,
  pending: DiamondMinusIcon,
  completed: CheckCircle2Icon,
  failed: XCircleIcon,
  skipped: SkipForwardIcon,
  orphaned: UnplugIcon,
  cancelled: BanIcon,
  unknown: CircleHelpIcon,
};

/** Falls back to `pending`, like `getStatusConfig`. */
export const getStatusIcon = (status: string): LucideIcon =>
  STATUS_ICONS[status as StatusKey] ?? STATUS_ICONS.pending;
