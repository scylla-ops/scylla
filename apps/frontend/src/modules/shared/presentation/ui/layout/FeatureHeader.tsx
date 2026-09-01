import type { ReactNode } from 'react';
import { useState } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { plural } from '@lingui/core/macro';
import { Button } from '@shadcn';
import { Trash } from 'lucide-react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { cn } from '@shared/presentation/utils';
import { toast } from 'sonner';

interface FeatureHeaderProps {
  count?: number;
  label: ReactNode;
  underLabel?: ReactNode;
  pluralLabel?: ReactNode;
  selectedCount?: number;
  /** True when every selectable row is already selected — hides the "Select all" button. */
  allSelected?: boolean;
  onSelectAll?: () => void;
  onClearSelection?: () => void;
  onDeleteSelection?: () => Promise<void> | void;
  onNew?: () => void;
  newLabel?: ReactNode;
  /** When false, the "New" button is shown disabled with {@link newDeniedReason}. */
  canNew?: boolean;
  newDeniedReason?: ReactNode;
  /** When false, the bulk-delete button is shown disabled with {@link deleteDeniedReason}. */
  canDelete?: boolean;
  deleteDeniedReason?: ReactNode;
  extraActions?: ReactNode;
  /** Shows a pulsing "new feature" dot next to the title. */
  isNew?: boolean;
}

export const FeatureHeader = ({
  count,
  label,
  pluralLabel,
  selectedCount = 0,
  allSelected = false,
  onSelectAll,
  onClearSelection,
  onDeleteSelection,
  onNew,
  newLabel,
  canNew = true,
  newDeniedReason,
  canDelete = true,
  deleteDeniedReason,
  extraActions,
  underLabel,
  isNew = false,
}: FeatureHeaderProps) => {
  const displayLabel = count && count > 1 ? (pluralLabel ?? label) : label;
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

  return (
    <div className={'flex flex-row items-end justify-between w-full'}>
      <div className={'flex flex-col gap-2'}>
        <div className={'flex flex-row gap-4'}>
          <div className='flex items-baseline gap-2'>
            <h1 className='text-3xl font-bold tracking-tight'>
              {count !== undefined && <span className='text-primary mr-2 '>{count}</span>}
              <span className='text-foreground'>{displayLabel}</span>
              {isNew && (
                <span className='relative inline-flex ml-2 h-2.5 w-2.5 align-middle'>
                  <span className='absolute inline-flex h-full w-full animate-ping rounded-full bg-primary opacity-75' />
                  <span className='relative inline-flex h-2.5 w-2.5 rounded-full bg-primary' />
                </span>
              )}
            </h1>
            {count !== undefined && (
              <span className='text-sm text-muted-foreground font-medium'>
                <Trans>in total</Trans>
              </span>
            )}
          </div>
        </div>
        {underLabel && <>{underLabel}</>}
      </div>

      <div className={'flex items-center justify-end gap-2'}>
        {onSelectAll && !allSelected && !!count && (
          <Button variant={'outline'} onClick={onSelectAll}>
            <Trans>Select all</Trans>
          </Button>
        )}
        {selectedCount > 0 && onClearSelection && (
          <Button variant={'outline'} onClick={onClearSelection}>
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
        {extraActions}
        {onNew &&
          (canNew ? (
            <Button onClick={onNew}>{newLabel ?? <Trans>New {label}</Trans>}</Button>
          ) : (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className='inline-flex'>
                  <Button disabled className='pointer-events-none'>
                    {newLabel ?? <Trans>New {label}</Trans>}
                  </Button>
                </span>
              </TooltipTrigger>
              <TooltipContent>
                <p>{newDeniedReason ?? <Trans>You don't have permission to do this.</Trans>}</p>
              </TooltipContent>
            </Tooltip>
          ))}
      </div>
      {onDeleteSelection && (
        <ConfirmOperationAlertDialog
          onContinue={handleDelete}
          open={deleteDialogOpen}
          onOpenChange={setDeleteDialogOpen}
        />
      )}
    </div>
  );
};
