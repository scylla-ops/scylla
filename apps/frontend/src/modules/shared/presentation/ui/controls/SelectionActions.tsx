import type { ReactNode } from 'react';
import { useState } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { plural } from '@lingui/core/macro';
import { Button } from '@shadcn';
import { Trash } from 'lucide-react';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { cn } from '@shared/presentation/utils';
import { toast } from 'sonner';

export interface SelectionActionsProps {
  selectedCount?: number;
  /** True when every selectable row is already selected — hides "Select all". */
  allSelected?: boolean;
  onSelectAll?: () => void;
  onClearSelection?: () => void;
  onDeleteSelection?: () => Promise<void> | void;
  /** When false, bulk delete is shown disabled with {@link deleteDeniedReason}. */
  canDelete?: boolean;
  deleteDeniedReason?: ReactNode;
  /** Number of selectable rows — "Select all" is pointless when there are none. */
  selectableCount?: number;
}

/**
 * Bulk actions over the current selection: clear, delete (behind a confirmation) and
 * select all. Rendered by `FeatureHeader` and by `DataTable`'s toolbar, which is why it
 * owns the confirm dialog and the success toast rather than either of them.
 *
 * "Select all" comes last on purpose: it is the one action that is always available, so
 * anchoring it at the end keeps it still while the transient ones appear beside it.
 */
export const SelectionActions = ({
  selectedCount = 0,
  allSelected = false,
  onSelectAll,
  onClearSelection,
  onDeleteSelection,
  canDelete = true,
  deleteDeniedReason,
  selectableCount,
}: SelectionActionsProps) => {
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const { t } = useLingui();

  const handleDelete = async () => {
    try {
      setDeleteDialogOpen(false);
      await onDeleteSelection?.();
      // Labels are ReactNode, so the entity name can't be spliced into a
      // translatable sentence — the confirmation stays deliberately generic.
      toast.success(
        t`${plural(selectedCount, { one: '# item deleted', other: '# items deleted' })}`,
      );
    } catch {
      // Toast shown by the global MutationCache onError handler.
      setDeleteDialogOpen(false);
    }
  };

  const canSelectAll = selectableCount === undefined ? true : selectableCount > 0;

  return (
    <>
      {onSelectAll && !allSelected && canSelectAll && (
        <Button variant='outline' onClick={onSelectAll}>
          <Trans>Select all</Trans>
        </Button>
      )}

      {selectedCount > 0 && onClearSelection && (
        <Button variant='outline' onClick={onClearSelection}>
          <Trans>Clear</Trans>
        </Button>
      )}

      {selectedCount > 0 && onDeleteSelection && (
        <Tooltip>
          <TooltipTrigger asChild>
            <span className='inline-flex'>
              <Button
                size='icon'
                variant='destructive'
                disabled={!canDelete}
                onClick={() => setDeleteDialogOpen(true)}
                className={cn(
                  'h-9 w-9 cursor-pointer transition-all hover:scale-110',
                  !canDelete && 'pointer-events-none',
                )}
              >
                <Trash className='size-4' />
              </Button>
            </span>
          </TooltipTrigger>
          <TooltipContent>
            <p>
              {canDelete ? (
                <Trans>Delete</Trans>
              ) : (
                (deleteDeniedReason ?? <Trans>You don't have permission to do this.</Trans>)
              )}
            </p>
          </TooltipContent>
        </Tooltip>
      )}

      {onDeleteSelection && (
        <ConfirmOperationAlertDialog
          onContinue={handleDelete}
          open={deleteDialogOpen}
          onOpenChange={setDeleteDialogOpen}
        />
      )}
    </>
  );
};
