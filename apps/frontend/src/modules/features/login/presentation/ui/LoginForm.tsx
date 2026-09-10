import { Trans, useLingui } from '@lingui/react/macro';
import { Button } from '@/modules/shared/presentation/ui/shadcn';
import { ScyllaForm } from '@shared/presentation/ui/forms/ScyllaForm.tsx';
import { FormItemType } from '@shared/presentation/structs/scylla-form.struct.ts';
import { useMemo } from 'react';

type LoginFormProps = {
  handleSubmit: (login: string, password: string) => void;
  isPending?: boolean;
};

export const LoginForm = ({ handleSubmit, isPending = false }: LoginFormProps) => {
  const { t } = useLingui();

  // `as const` keeps the ids literal, which is what types the submitted values.
  const formItems = useMemo(
    () =>
      [
        {
          id: 'username',
          label: <Trans>Username</Trans>,
          placeholder: t`username`,
          type: FormItemType.Input,
          inputType: 'text',
        },
        {
          id: 'password',
          label: <Trans>Password</Trans>,
          placeholder: t`••••••••`,
          type: FormItemType.Input,
          inputType: 'password',
        },
      ] as const,
    [t],
  );

  return (
    <ScyllaForm
      items={formItems}
      className='gap-4'
      onSubmit={values => handleSubmit(values.username, values.password)}
      isPending={isPending}
      footer={({ isValid, isPending }) => (
        <Button type='submit' className='mt-2 w-full' disabled={!isValid || isPending}>
          <Trans>Login</Trans>
        </Button>
      )}
    />
  );
};
