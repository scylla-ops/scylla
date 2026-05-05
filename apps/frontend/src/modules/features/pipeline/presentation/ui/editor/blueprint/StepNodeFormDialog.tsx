import { FormDialog } from '@shared/presentation/ui';
import { type FormChange, type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { Trans, useLingui } from '@lingui/react/macro';
import type { PipelineNodeData } from '@/modules/features/pipeline/presentation/utils/blueprint-converter.ts';

interface StepNodeFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** If provided, the dialog is in edit mode with pre-filled values. */
  editingNode?: PipelineNodeData | null;
  onAdd: (nodeId: string, command: string, args: string[]) => void;
  onEdit: (originalId: string, nodeId: string, command: string, args: string[]) => void;
}

export function StepNodeFormDialog({ open, onOpenChange, editingNode, onAdd, onEdit }: StepNodeFormDialogProps) {
  const { t } = useLingui();
  const isEditMode = !!editingNode;

  const items: FormItem[] = [
    {
      id: 'nodeId',
      label: t`Node ID`,
      placeholder: t`e.g., build`,
      type: FormItemType.Input,
      inputType: 'text',
      defaultValue: editingNode?.id ?? '',
    },
    {
      id: 'command',
      label: t`Command`,
      placeholder: t`e.g., cargo`,
      type: FormItemType.Input,
      inputType: 'text',
      defaultValue: editingNode?.command ?? '',
    },
    {
      id: 'args',
      label: t`Arguments (space-separated)`,
      placeholder: t`e.g., build --release`,
      type: FormItemType.Input,
      inputType: 'text',
      defaultValue: editingNode?.args.join(' ') ?? '',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const nodeId = values.find(v => v.id === 'nodeId')?.value ?? '';
    const command = values.find(v => v.id === 'command')?.value ?? '';
    const argsStr = values.find(v => v.id === 'args')?.value ?? '';
    const args = argsStr.trim() ? argsStr.trim().split(/\s+/) : [];

    if (!nodeId.trim() || !command.trim()) return;

    if (isEditMode) {
      onEdit(editingNode.id, nodeId.trim(), command.trim(), args);
    } else {
      onAdd(nodeId.trim(), command.trim(), args);
    }
    onOpenChange(false);
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={isEditMode ? <Trans>Edit node</Trans> : <Trans>Add a new node</Trans>}
      description={
        isEditMode
          ? <Trans>Modify the node properties.</Trans>
          : <Trans>Define a new pipeline step. You can connect it to other nodes by dragging edges.</Trans>
      }
      items={items}
      isPending={false}
      submitLabel={isEditMode ? <Trans>Save</Trans> : <Trans>Add Node</Trans>}
      onSubmit={handleSubmit}
    />
  );
}

