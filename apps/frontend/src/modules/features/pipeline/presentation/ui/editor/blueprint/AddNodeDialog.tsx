import { FormDialog } from '@shared/presentation/ui';
import { type FormChange, type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { Trans, useLingui } from '@lingui/react/macro';

interface AddNodeDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  onAdd: (nodeId: string, command: string, args: string[]) => void;
}

export function AddNodeDialog({ open, setOpen, onAdd }: AddNodeDialogProps) {
  const { t } = useLingui();

  const items: FormItem[] = [
    {
      id: 'nodeId',
      label: t`Node ID`,
      placeholder: t`e.g., build`,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'command',
      label: t`Command`,
      placeholder: t`e.g., cargo`,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'args',
      label: t`Arguments (space-separated)`,
      placeholder: t`e.g., build --release`,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const nodeId = values.find(v => v.id === 'nodeId')?.value ?? '';
    const command = values.find(v => v.id === 'command')?.value ?? '';
    const argsStr = values.find(v => v.id === 'args')?.value ?? '';
    const args = argsStr.trim() ? argsStr.trim().split(/\s+/) : [];

    if (!nodeId.trim() || !command.trim()) return;

    onAdd(nodeId.trim(), command.trim(), args);
    setOpen(false);
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={setOpen}
      title={<Trans>Add a new node</Trans>}
      description={<Trans>Define a new pipeline step. You can connect it to other nodes by dragging edges.</Trans>}
      items={items}
      isPending={false}
      submitLabel={<Trans>Add Node</Trans>}
      onSubmit={handleSubmit}
    />
  );
}

