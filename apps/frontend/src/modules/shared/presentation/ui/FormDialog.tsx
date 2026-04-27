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
import type { FormChange, FormItem } from '@shared/presentation/models/scylla-form.model.ts';
import { ScyllaForm } from '@shared/presentation/ui/ScyllaForm.tsx';

interface FormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  description?: ReactNode;
  items: FormItem[];
  isPending?: boolean;
  submitLabel?: ReactNode;
  pendingLabel?: ReactNode;
  onSubmit: (values: FormChange[]) => void;
}

export function FormDialog({
  open,
  onOpenChange,
  title,
  description,
  items,
  isPending = false,
  submitLabel,
  pendingLabel,
  onSubmit,
}: FormDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
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
              <Button
                type='button'
                variant='outline'
                onClick={() => onOpenChange(false)}
                disabled={isPending}
              >
                <Trans>Cancel</Trans>
              </Button>
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
