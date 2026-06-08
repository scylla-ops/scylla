import { FormDialog } from '@shared/presentation/ui';
import { createSecretsItems } from '@/modules/features/secret/presentation/utils/createSecretItems.ts';
import { useCreateSecret } from '@/modules/features/secret/presentation/hooks/use-secrets.ts';
import type { FormChange } from '@shared/presentation/models/scylla-form.model.ts';

interface CreateSecretDialogProps {
  isOpen: boolean;
  setOpen: (open: boolean) => void;
  projectId: string;
}

export const CreateSecretDialog = ({ projectId, isOpen, setOpen }: CreateSecretDialogProps) => {
  const createSecret = useCreateSecret(projectId);

  const handleSubmit = (values: FormChange[]) => {
    const name = values.find(value => value.id == 'name')?.value as string;
    const description = values.find(value => value.id == 'description')?.value as string;
    const value = values.find(value => value.id == 'value')?.value as string;
    if (!name?.trim() || !value?.trim()) return;

    createSecret.mutate({ name, description: description?.trim() || '', value });
    setOpen(false);
  };
  return (
    <FormDialog
      open={isOpen}
      onOpenChange={setOpen}
      title={'Create secret'}
      items={createSecretsItems()}
      onSubmit={handleSubmit}
    />
  );
};
