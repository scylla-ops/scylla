import { useCreateUser } from '@/modules/features/user/presentation/hooks/use-create-user.ts';
import { Trans, useLingui } from '@lingui/react/macro';
import {
  type FormChange,
  type FormItem,
  FormItemType,
} from '@shared/presentation/structs/scylla-form.struct.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { FormDialog } from '@shared/presentation/ui';

interface AddUserDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
}

export function AddUserDialog({ open, setOpen }: AddUserDialogProps) {
  const { t, i18n } = useLingui();
  const createUser = useCreateUser();

  const items: FormItem[] = [
    {
      id: 'username',
      label: t`Username`,
      placeholder: t`e.g., john.doe`,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'password',
      label: t`Password`,
      placeholder: t`Enter a password`,
      type: FormItemType.Input,
      inputType: 'password',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const username = values.find(v => v.id === 'username')?.value;
    const password = values.find(v => v.id === 'password')?.value;

    if (!username?.trim() || !password?.trim()) {
      toast.error(i18n._(ToastMessages.USER_CREDENTIALS_REQUIRED_ERROR));
      return;
    }

    createUser.mutate({ username, password }, { onSuccess: () => setOpen(false) });
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={setOpen}
      title={<Trans>Create a new user</Trans>}
      description={<Trans>Enter a username and password for the new user.</Trans>}
      items={items}
      isPending={createUser.isPending}
      submitLabel={<Trans>Create User</Trans>}
      onSubmit={handleSubmit}
    />
  );
}
