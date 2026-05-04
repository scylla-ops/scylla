import type { ReactNode } from 'react';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { Button } from '@shadcn';
import { Trash } from 'lucide-react';
import { ConfirmOperationAlertDialog } from '@shared/presentation/ui/ConfirmOperationAlertDialog.tsx';

interface FeatureHeaderProps {
  count: number;
  label: string;
  pluralLabel?: string;
  selectedCount?: number;
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
  onClearSelection,
  onDeleteSelection,
  onNew,
  newLabel,
  extraActions,
}: FeatureHeaderProps) => {
  const displayLabel = count > 1 ? (pluralLabel ?? `${label}s`) : label;
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  const handleDelete = async () => {
    try {
      await onDeleteSelection?.();
    } finally {
      setDeleteDialogOpen(false);
    }
  };

  return (
    <div className={'flex flex-row items-center justify-between w-full'}>
      <div className='flex items-baseline gap-2'>
        <h1 className='text-3xl font-bold tracking-tight'>
          <span className='text-primary'>{count}</span>{' '}
          <span className='text-foreground'>{displayLabel}</span>
        </h1>
        <span className='text-sm text-muted-foreground font-medium'>
          <Trans>in total</Trans>
        </span>
      </div>
      <div className={'flex items-center justify-end gap-2'}>
        {selectedCount > 0 && onClearSelection && (
          <Button variant={'outline'} onClick={onClearSelection}>
            <Trans>Clear</Trans>
          </Button>
        )}
        {selectedCount > 0 && onDeleteSelection && (
          <Button
            size='icon'
            variant='destructive'
            onClick={() => setDeleteDialogOpen(true)}
            className='h-9 w-9'
          >
            <Trash className='size-4' />
          </Button>
        )}
        {extraActions}
        {onNew && (
          <Button onClick={onNew}>
            {newLabel ?? <Trans>New {label}</Trans>}
          </Button>
        )}
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
