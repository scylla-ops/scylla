import { Trans, useLingui } from '@lingui/react/macro';
import { FormDialog } from '@shared/presentation/ui';
import {
  type FormChange,
  type FormItem,
  FormItemType,
} from '@shared/presentation/structs/scylla-form.struct.ts';
import { useUpdateOrganization } from '@/modules/features/organization/presentation/hooks/use-update-organization.ts';

interface EditOrganizationDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  organization: { id: string; name: string; description?: string };
}

export function EditOrganizationDialog({
  open,
  setOpen,
  organization,
}: EditOrganizationDialogProps) {
  const { t } = useLingui();
  const updateOrganization = useUpdateOrganization();

  const items: FormItem[] = [
    {
      id: 'name',
      label: t`Organization name`,
      placeholder: organization.name,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'description',
      label: t`Description`,
      placeholder: organization.description || t`Add a description...`,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value;
    const description = values.find(v => v.id === 'description')?.value;

    updateOrganization.mutate(
      {
        organizationId: organization.id,
        name: name?.trim() || undefined,
        description: description?.trim() || undefined,
      },
      { onSuccess: () => setOpen(false) },
    );
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={setOpen}
      title={<Trans>Edit organization</Trans>}
      description={<Trans>Update the organization name and description.</Trans>}
      items={items}
      isPending={updateOrganization.isPending}
      submitLabel={<Trans>Save</Trans>}
      pendingLabel={<Trans>Saving...</Trans>}
      onSubmit={handleSubmit}
    />
  );
}
