import type { ReactNode } from 'react';
import { Button } from '@shadcn';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@shadcn/dialog.tsx';
import { Trans } from '@lingui/react/macro';
import type { FormItem, FormValues } from '@shared/presentation/structs/scylla-form.struct.ts';
import { ScyllaForm } from '@shared/presentation/ui/forms/ScyllaForm.tsx';

interface FormDialogProps<TId extends string> {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  description?: ReactNode;
  items: readonly FormItem<TId>[];
  isPending?: boolean;
  submitLabel?: ReactNode;
  pendingLabel?: ReactNode;
  onSubmit: (values: FormValues<TId>) => void;
  hideCancel?: boolean;
}

export function FormDialog<TId extends string>({
  open,
  onOpenChange,
  title,
  description,
  items,
  isPending = false,
  submitLabel,
  pendingLabel,
  onSubmit,
  hideCancel = false,
}: FormDialogProps<TId>) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className={hideCancel ? '[&>button]:hidden' : ''}>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          {description && <DialogDescription>{description}</DialogDescription>}
        </DialogHeader>
        <ScyllaForm
          items={items}
          isPending={isPending}
          onSubmit={onSubmit}
          footer={({ isValid, isPending }) => (
            <DialogFooter>
              {!hideCancel && (
                <Button
                  type='button'
                  variant='outline'
                  onClick={() => onOpenChange(false)}
                  disabled={isPending}
                >
                  <Trans>Cancel</Trans>
                </Button>
              )}
              <Button type='submit' disabled={!isValid || isPending}>
                {isPending
                  ? (pendingLabel ?? <Trans>Creating...</Trans>)
                  : (submitLabel ?? <Trans>Create</Trans>)}
              </Button>
            </DialogFooter>
          )}
        />
      </DialogContent>
    </Dialog>
  );
}

export default FormDialog;
