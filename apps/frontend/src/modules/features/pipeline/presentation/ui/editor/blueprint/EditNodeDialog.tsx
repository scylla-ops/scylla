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
import type { PipelineNodeData } from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';

interface EditNodeDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  nodeData: PipelineNodeData | null;
  onSave: (originalId: string, nodeId: string, command: string, args: string[]) => void;
}

export function EditNodeDialog({ open, setOpen, nodeData, onSave }: EditNodeDialogProps) {
  const { t } = useLingui();
  const [nodeId, setNodeId] = useState('');
  const [command, setCommand] = useState('');
  const [argsStr, setArgsStr] = useState('');

  useEffect(() => {
    if (nodeData && open) {
      setNodeId(nodeData.id);
      setCommand(nodeData.command);
      setArgsStr(nodeData.args.join(' '));
    }
  }, [nodeData, open]);

  if (!nodeData) return null;

  const isValid = nodeId.trim().length > 0 && command.trim().length > 0;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValid) return;
    const args = argsStr.trim() ? argsStr.trim().split(/\s+/) : [];
    onSave(nodeData.id, nodeId.trim(), command.trim(), args);
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Edit node</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Modify the node properties.</Trans>
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className='space-y-4'>
          <FieldGroup>
            <Field className='gap-1'>
              <FieldLabel htmlFor='edit-nodeId'>{t`Node ID`}</FieldLabel>
              <Input
                id='edit-nodeId'
                value={nodeId}
                onChange={e => setNodeId(e.target.value)}
                autoFocus
              />
            </Field>
            <Field className='gap-1'>
              <FieldLabel htmlFor='edit-command'>{t`Command`}</FieldLabel>
              <Input id='edit-command' value={command} onChange={e => setCommand(e.target.value)} />
            </Field>
            <Field className='gap-1'>
              <FieldLabel htmlFor='edit-args'>{t`Arguments (space-separated)`}</FieldLabel>
              <Input
                id='edit-args'
                value={argsStr}
                onChange={e => setArgsStr(e.target.value)}
                placeholder={t`e.g., build --release`}
              />
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
