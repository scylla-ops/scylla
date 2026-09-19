/**
 * The Svelte port of the shadcn primitives.
 *
 * Ported on demand, not wholesale: a primitive lands here the phase its first
 * Svelte consumer does. `shadcn/` next door holds the React originals and is
 * **frozen** for the duration — bug fixes only — so the two cannot drift while
 * both are alive. Class strings are copied verbatim, which is why the design
 * survives the migration untouched.
 *
 * Deleted along with `shadcn/` in Phase 6.
 */
import {
  AlertDialog as AlertDialogPrimitive,
  Dialog as DialogPrimitive,
  Select as SelectPrimitive,
} from 'bits-ui';

export { default as Button } from './button.svelte';
// Not from the `.svelte` file: `tsc` only ever sees a component's default
// export, so anything a `.ts` needs to import must live in a `.ts`.
export { buttonVariants, type ButtonSize, type ButtonVariant } from './button-variants.ts';

export { default as Card } from './card.svelte';
export { default as CardAction } from './card-action.svelte';
export { default as CardContent } from './card-content.svelte';
export { default as CardDescription } from './card-description.svelte';
export { default as CardFooter } from './card-footer.svelte';
export { default as CardHeader } from './card-header.svelte';
export { default as CardTitle } from './card-title.svelte';

export { default as Input } from './input.svelte';
export { default as Skeleton } from './skeleton.svelte';

export { default as Tooltip } from './tooltip.svelte';
export { default as TooltipContent } from './tooltip-content.svelte';
export { default as TooltipTrigger } from './tooltip-trigger.svelte';

// The parts that carry no styling are bits-ui's own, aliased here rather than
// wrapped — `shadcn/dialog.tsx` does exactly the same with Radix. A wrapper
// whose whole body is `<Primitive {...rest} />` is a file to keep in sync for
// nothing.
export const Dialog = DialogPrimitive.Root;
export const DialogClose = DialogPrimitive.Close;
export const DialogPortal = DialogPrimitive.Portal;
export const DialogTrigger = DialogPrimitive.Trigger;
export { default as DialogContent } from './dialog-content.svelte';
export { default as DialogDescription } from './dialog-description.svelte';
export { default as DialogFooter } from './dialog-footer.svelte';
export { default as DialogHeader } from './dialog-header.svelte';
export { default as DialogOverlay } from './dialog-overlay.svelte';
export { default as DialogTitle } from './dialog-title.svelte';

export const AlertDialog = AlertDialogPrimitive.Root;
export const AlertDialogPortal = AlertDialogPrimitive.Portal;
export const AlertDialogTrigger = AlertDialogPrimitive.Trigger;
export { default as AlertDialogAction } from './alert-dialog-action.svelte';
export { default as AlertDialogCancel } from './alert-dialog-cancel.svelte';
export { default as AlertDialogContent } from './alert-dialog-content.svelte';
export { default as AlertDialogDescription } from './alert-dialog-description.svelte';
export { default as AlertDialogFooter } from './alert-dialog-footer.svelte';
export { default as AlertDialogHeader } from './alert-dialog-header.svelte';
export { default as AlertDialogOverlay } from './alert-dialog-overlay.svelte';
export { default as AlertDialogTitle } from './alert-dialog-title.svelte';

export { default as Checkbox } from './checkbox.svelte';

export { default as Avatar } from './avatar.svelte';
export { default as AvatarFallback } from './avatar-fallback.svelte';
export { default as AvatarImage } from './avatar-image.svelte';

export { default as TableBody } from './table-body.svelte';
export { default as TableCell } from './table-cell.svelte';
export { default as TableHead } from './table-head.svelte';
export { default as TableHeader } from './table-header.svelte';
export { default as TableRow } from './table-row.svelte';

export { default as Label } from './label.svelte';
export { default as Field } from './field.svelte';
export { default as FieldGroup } from './field-group.svelte';
export { default as FieldLabel } from './field-label.svelte';
export { fieldVariants, type FieldOrientation } from './field-variants.ts';

export const Select = SelectPrimitive.Root;
export { default as SelectContent } from './select-content.svelte';
export { default as SelectItem } from './select-item.svelte';
export { default as SelectTrigger } from './select-trigger.svelte';
export { default as SelectValue } from './select-value.svelte';

export { default as Pagination } from './pagination.svelte';
export { default as PaginationContent } from './pagination-content.svelte';
export { default as PaginationEllipsis } from './pagination-ellipsis.svelte';
export { default as PaginationItem } from './pagination-item.svelte';
export { default as PaginationLink } from './pagination-link.svelte';
export { default as PaginationNext } from './pagination-next.svelte';
export { default as PaginationPrevious } from './pagination-previous.svelte';
