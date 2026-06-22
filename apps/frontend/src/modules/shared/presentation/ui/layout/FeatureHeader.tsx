import type { ReactNode } from 'react';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { Button } from '@shadcn';
import { Trash } from 'lucide-react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/feedback/ConfirmOperationAlertDialog.tsx';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
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
  extraActions?: ReactNode;
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
  extraActions,
  underLabel,
}: FeatureHeaderProps) => {
  const displayLabel = count && count > 1 ? (pluralLabel ?? label) : label;
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const handleDelete = async () => {
    try {
      setDeleteDialogOpen(false);
      await onDeleteSelection?.();
      const itemLabel = typeof label === 'string' ? label : 'item';
      toast.success(`${selectedCount} ${itemLabel}s deleted`);
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
              <Button
                size='icon'
                variant='destructive'
                onClick={() => setDeleteDialogOpen(true)}
                className='h-9 w-9 cursor-pointer transition-all hover:scale-110'
              >
                <Trash className='size-4' />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>
                <Trans>Delete</Trans>
              </p>
            </TooltipContent>
          </Tooltip>
        )}
        {extraActions}
        {onNew && <Button onClick={onNew}>{newLabel ?? <Trans>New {label}</Trans>}</Button>}
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
