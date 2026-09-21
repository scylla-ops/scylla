import BanIcon from '@lucide/svelte/icons/ban';
import CheckCircle2Icon from '@lucide/svelte/icons/check-circle-2';
import CircleHelpIcon from '@lucide/svelte/icons/circle-help';
import DiamondMinusIcon from '@lucide/svelte/icons/diamond-minus';
import Loader2Icon from '@lucide/svelte/icons/loader-2';
import SkipForwardIcon from '@lucide/svelte/icons/skip-forward';
import UnplugIcon from '@lucide/svelte/icons/unplug';
import XCircleIcon from '@lucide/svelte/icons/x-circle';
import { STATUS_CONFIG, type StatusKey } from '@shared/utils/status-config.ts';
import type { LucideIcon } from '../icon.ts';

/**
 * The icon half of {@link STATUS_CONFIG}, in Svelte components.
 *
 * Split out of `status-config.ts` because that file is `utils/` and must hold
 * no framework: the icon was the one field in it that was a React component,
 * and it is also the one field that cannot be shared between the two halves of
 * the migration. Everything else about a status — its label, its colours —
 * still comes from the single table next door, so the two sides cannot drift on
 * anything that matters.
 *
 * Keyed by {@link StatusKey} rather than merged into the config object: a
 * `Record` the compiler checks for exhaustiveness is what makes a status added
 * to the union fail here instead of rendering nothing.
 */
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

/**
 * The icon for a status string, falling back to `pending` — the same fallback
 * `getStatusConfig` applies, so an unrecognised status gets a matching pair.
 */
export const getStatusIcon = (status: string): LucideIcon =>
  STATUS_ICONS[status as StatusKey] ?? STATUS_ICONS.pending;
