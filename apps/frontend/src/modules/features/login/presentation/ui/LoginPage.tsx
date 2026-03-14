import { LoginForm } from '@/modules/features/login/presentation/ui/LoginForm.tsx';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shadcn';
import LogoScylla from '@/assets/logo_scylla.png';
import { Trans } from '@lingui/react/macro';
import { useLogin } from '@/modules/features/login/presentation/hooks/useLogin.ts';
import { type FormEvent } from 'react';

export const LoginPage = () => {
  const { mutate: login, error, isPending, isSuccess } = useLogin();

  const handleSubmit = (e: FormEvent, loginValue: string, passwordValue: string) => {
    e.preventDefault();
    login({ login: loginValue, password: passwordValue });
  };

  if (isPending || isSuccess) return <p>Loading... (todo change that)</p>;

  return (
    <div className={'flex items-center flex-col'}>
      <div className='flex flex-row items-center space-x-1 mb-4 mt-12 pr-6'>
        <img src={LogoScylla} alt='logo' className='object-contain w-32 h-32' />
        <h1 className='text-4xl font-bold'>Scylla</h1>
      </div>
      <Card className='w-full max-w-sm'>
        <CardHeader>
          <CardTitle>
            <Trans>Login to your account</Trans>
          </CardTitle>
          <CardDescription>
            <Trans>Enter your email below to login to your account</Trans>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <LoginForm handleSubmit={handleSubmit} />
          {error ? <div>{error.message}</div> : <></>}
        </CardContent>
      </Card>
    </div>
  );
};

export default LoginPage;
