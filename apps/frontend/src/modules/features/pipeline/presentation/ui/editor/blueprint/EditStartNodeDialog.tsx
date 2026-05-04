import { useState, useEffect } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { Button, Input } from '@shadcn';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@shadcn/dialog.tsx';
import { Field, FieldGroup, FieldLabel } from '@shadcn/field.tsx';

interface EditStartNodeDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  name: string;
  onSave: (name: string) => void;
}

export function EditStartNodeDialog({ open, setOpen, name, onSave }: EditStartNodeDialogProps) {
  const { t } = useLingui();
  const [value, setValue] = useState('');

  useEffect(() => {
    if (open) setValue(name);
  }, [name, open]);

  const isValid = value.trim().length > 0;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValid) return;
    onSave(value.trim());
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle><Trans>Pipeline name</Trans></DialogTitle>
          <DialogDescription><Trans>Set the name of the pipeline.</Trans></DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className='space-y-4'>
          <FieldGroup>
            <Field className='gap-1'>
              <FieldLabel htmlFor='edit-pipeline-name'>{t`Name`}</FieldLabel>
              <Input id='edit-pipeline-name' value={value} onChange={e => setValue(e.target.value)} autoFocus />
            </Field>
          </FieldGroup>
          <DialogFooter>
            <Button type='button' variant='outline' onClick={() => setOpen(false)}>
              <Trans>Cancel</Trans>
            </Button>
            <Button type='submit' disabled={!isValid}>
              <Trans>Save</Trans>
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

