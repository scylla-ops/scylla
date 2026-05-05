import { useCreateOrganization } from '@/modules/features/organization/presentation/hooks/useCreateOrganization.ts';
import { Trans, useLingui } from '@lingui/react/macro';
import { FormDialog } from '@shared/presentation/ui';
import {
  type FormChange,
  type FormItem,
  FormItemType,
} from '@shared/presentation/models/scylla-form.model.ts';

interface AddOrganizationDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  hideCancel?: boolean;
}

export function AddOrganizationDialog({
  open,
  setOpen,
  hideCancel = false,
}: AddOrganizationDialogProps) {
  const { t } = useLingui();
  const createOrganization = useCreateOrganization();

  const items: FormItem[] = [
    {
      id: 'name',
      label: t`Organization name`,
      placeholder: t`e.g., My Organization`,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'description',
      label: t`Description`,
      placeholder: t`e.g., Our company's main organization`,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value;
    const description = values.find(v => v.id === 'description')?.value;
    if (!name?.trim()) return;

    createOrganization.mutate(
      { name, description: description?.trim() || undefined },
      {
        onSuccess: () => setOpen(false),
      },
    );
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={setOpen}
      title={<Trans>Create a new organization</Trans>}
      description={
        <Trans>
          Enter a name and description for your new organization. You can change these later in
          settings.
        </Trans>
      }
      items={items}
      isPending={createOrganization.isPending}
      submitLabel={<Trans>Create Organization</Trans>}
      onSubmit={handleSubmit}
      hideCancel={hideCancel}
    />
  );
}
