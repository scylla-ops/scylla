import { useCreateOrganization } from '@/modules/features/organization/presentation/hooks/useCreateOrganization.ts';
import { Trans } from '@lingui/react/macro';
import { FormDialog } from '@shared/presentation/ui';
import { type FormChange } from '@shared/presentation/models/scylla-form.model.ts';
import { createOrganizationItems } from '@/modules/features/organization/presentation/utils/create-organization-form-items.ts';

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
  const createOrganization = useCreateOrganization();

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
      items={createOrganizationItems()}
      isPending={createOrganization.isPending}
      submitLabel={<Trans>Create Organization</Trans>}
      onSubmit={handleSubmit}
      hideCancel={hideCancel}
    />
  );
}
