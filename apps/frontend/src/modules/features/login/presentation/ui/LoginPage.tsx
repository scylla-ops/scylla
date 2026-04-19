import { LoginForm } from '@/modules/features/login/presentation/ui/LoginForm.tsx';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/modules/shared/presentation/ui/shadcn';
import LogoScylla from '@/assets/logo_scylla.png';
import { Trans } from '@lingui/react/macro';
import { useLogin } from '@/modules/features/login/presentation/hooks/useLogin.ts';
import { type FormEvent } from 'react';
import { Loader2 } from 'lucide-react';

export const LoginPage = () => {
  const { mutate: login, isPending, isSuccess } = useLogin();

  const handleSubmit = (e: FormEvent, loginValue: string, passwordValue: string) => {
    e.preventDefault();
    login({ login: loginValue, password: passwordValue });
  };

  if (isPending || isSuccess)
    return (
      <div className='flex flex-col items-center justify-center h-screen gap-4'>
        <img src={LogoScylla} alt='logo' className='w-20 h-20 object-contain' />
        <Loader2 className='h-6 w-6 animate-spin text-muted-foreground' />
      </div>
    );

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
        </CardContent>
      </Card>
    </div>
  );
};

export default LoginPage;
