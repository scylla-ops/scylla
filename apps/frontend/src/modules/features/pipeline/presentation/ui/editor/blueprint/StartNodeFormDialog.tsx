import { FormDialog } from '@shared/presentation/ui';
import { type FormChange, type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { Trans, useLingui } from '@lingui/react/macro';

interface StartNodeFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  currentName: string;
  onSave: (name: string) => void;
}

export function StartNodeFormDialog({ open, onOpenChange, currentName, onSave }: StartNodeFormDialogProps) {
  const { t } = useLingui();

  const items: FormItem[] = [
    {
      id: 'name',
      label: t`Name`,
      placeholder: t`e.g., my-pipeline`,
      type: FormItemType.Input,
      inputType: 'text',
      defaultValue: currentName,
      pattern: '^\\S+$',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value ?? '';
    if (!name.trim()) return;
    onSave(name.trim());
    onOpenChange(false);
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={onOpenChange}
      title={<Trans>Pipeline name</Trans>}
      description={<Trans>Set the name of the pipeline.</Trans>}
      items={items}
      isPending={false}
      submitLabel={<Trans>Save</Trans>}
      onSubmit={handleSubmit}
    />
  );
}

