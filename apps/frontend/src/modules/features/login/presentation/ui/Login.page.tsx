import { LoginForm } from '@/modules/features/login/presentation/ui/LoginForm.tsx';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/modules/shared/presentation/ui/shadcn';
import LogoScylla from '@/assets/logo_scylla.png';
import LogoScyllaDark from '@/assets/Scylla_Beta_Logo_Black_Theme.png';
import { Trans } from '@lingui/react/macro';
import { useLogin } from '@/modules/features/login/presentation/hooks/use-login.ts';
import { type FormEvent } from 'react';
import { Loader2 } from 'lucide-react';

/**
 * The wordmark is flat black, unreadable on the dark background — the dark
 * variant is the white cut of the same logo.
 *
 * Swapped by CSS rather than by reading `resolvedTheme`: next-themes only knows
 * the theme after mount, so a JS swap would paint the wrong logo first and
 * flash. The `.dark` class is on <html> before first paint, so this is right
 * from the start.
 */
const ScyllaLogo = ({ className }: { className: string }) => (
  <>
    <img src={LogoScylla} alt='Scylla' className={`${className} dark:hidden`} />
    <img src={LogoScyllaDark} alt='Scylla' className={`${className} hidden dark:block`} />
  </>
);

export const LoginPage = () => {
  const { mutate: login, isPending, isSuccess } = useLogin();

  const handleSubmit = (e: FormEvent, loginValue: string, passwordValue: string) => {
    e.preventDefault();
    login({ login: loginValue, password: passwordValue });
  };

  if (isPending || isSuccess)
    return (
      <div className='flex flex-col items-center justify-center h-screen gap-4'>
        <ScyllaLogo className='w-20 h-20 object-contain' />
        <Loader2 className='h-6 w-6 animate-spin text-muted-foreground' />
      </div>
    );

  return (
    <div className={'flex items-center flex-col'}>
      <ScyllaLogo className='w-1/6 h-1/6' />
      <Card className='w-full max-w-sm'>
        <CardHeader>
          <CardTitle>
            <Trans>Login to your account</Trans>
          </CardTitle>
          <CardDescription>
            <Trans>Enter your username below to login to your account</Trans>
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
