<script lang="ts">
  import { i18n } from '@lingui/core';
  import { createMutation } from '@platform/query';
  import {
    FormDialog,
    FormItemType,
    type FormItem,
    type FormValues,
  } from '@shared/presentation/ui-svelte';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { userMutations } from '../../user.queries.ts';
  import { userMessages } from '../user.messages.ts';

  interface Props {
    open: boolean;
    setOpen: (open: boolean) => void;
  }

  let { open, setOpen }: Props = $props();

  const createUser = createMutation(() => userMutations.create());

  const items: readonly FormItem<'username' | 'password'>[] = $derived([
    {
      id: 'username',
      label: t(userMessages.username),
      placeholder: t(userMessages.usernamePlaceholder),
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'password',
      label: t(userMessages.password),
      placeholder: t(userMessages.passwordPlaceholder),
      type: FormItemType.Input,
      inputType: 'password',
    },
  ]);

  const handleSubmit = ({ username, password }: FormValues<'username' | 'password'>) => {
    if (!username.trim() || !password.trim()) {
      toast.error(i18n._(ToastMessages.USER_CREDENTIALS_REQUIRED_ERROR));
      return;
    }

    // Closed from the mutation's own callback, not before it: unlike the secret
    // dialog, this one stays up and pending until the user actually exists.
    createUser.mutate({ username, password }, { onSuccess: () => setOpen(false) });
  };
</script>

<FormDialog
  {open}
  onOpenChange={setOpen}
  title={t(userMessages.createTitle)}
  description={t(userMessages.createDescription)}
  {items}
  isPending={createUser.isPending}
  submitLabel={t(userMessages.createSubmit)}
  onSubmit={handleSubmit}
/>
