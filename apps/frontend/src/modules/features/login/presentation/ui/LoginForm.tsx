import { Trans, useLingui } from '@lingui/react/macro';
import { Button, Input, Label } from '@/modules/shared/presentation/ui/shadcn';
import { useState } from 'react';
import * as React from 'react';

type LoginFormProps = {
  handleSubmit: (e: React.FormEvent, login: string, password: string) => void;
};

//TODO: use ScyllaForm instead of this
export const LoginForm = ({ handleSubmit }: LoginFormProps) => {
  const { t } = useLingui();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');

  return (
    <form
      onSubmit={e => handleSubmit(e, email, password)}
      className='flex flex-col gap-4 w-auto mx-auto'
    >
      <div>
        <Label htmlFor='email'><Trans>Email</Trans></Label>
        <Input
          id='email'
          type='text'
          value={email}
          onChange={e => setEmail(e.target.value)}
          placeholder={t`you@example.com`}
          required
        />
      </div>

      <div>
        <Label htmlFor='password'><Trans>Password</Trans></Label>
        <Input
          id='password'
          type='password'
          value={password}
          onChange={e => setPassword(e.target.value)}
          placeholder={t`••••••••`}
          required
        />
      </div>

      <Button type='submit' className='mt-2'>
        <Trans>Login</Trans>
      </Button>
    </form>
  );
};
