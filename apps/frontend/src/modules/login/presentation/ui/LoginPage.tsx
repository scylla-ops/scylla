import { LoginForm } from '@/modules/login/presentation/ui/LoginForm.tsx';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/modules/core/presentation/ui/shadcn';
import LogoScylla from '@/assets/logo_scylla.png';
import { Trans } from '@lingui/react/macro';
import { useLogin } from '@/modules/login/presentation/hooks/login.ts';
import { type FormEvent } from 'react';
import { useToken } from '@/modules/login/presentation/store/tokenStore.ts';

export const LoginPage = () => {
  const { mutate: login, status, error } = useLogin();
  const setToken = useToken(state => state.setToken);

  const handleSubmit = (e: FormEvent, loginValue: string, passwordValue: string) => {
    e.preventDefault();

    login(
      { login: loginValue, password: passwordValue },
      {
        onSuccess: res => {
          if (!res.ok) {
            console.log('Error logging in!', res.error);
          } else {
            setToken(res.value);
            console.log('Logged in!', res.value);
          }
        },
      },
    );
  };

  return (
    <div className={'flex items-center h-screen flex-col'}>
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
        </CardContent>
      </Card>
    </div>
  );
};

export default LoginPage;
