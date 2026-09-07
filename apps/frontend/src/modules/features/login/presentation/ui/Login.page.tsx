import { LoginForm } from '@/modules/features/login/presentation/ui/LoginForm.tsx';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/modules/shared/presentation/ui/shadcn';
import LogoScylla from '@/assets/logo_scylla.png';
import LogoScyllaDark from '@/assets/logo_scylla_dark.png';
import { Trans } from '@lingui/react/macro';
import { useLogin } from '@/modules/features/login/presentation/hooks/use-login.ts';
import { type FormEvent } from 'react';
import { motion } from 'framer-motion';
import scyllaLogoDark from '@/assets/logo_scylla_dark.png';
import scyllaLogo from '@/assets/logo_scylla.png';
import { useTheme } from 'next-themes';

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
  const isDarkTheme = useTheme().theme === 'dark';

  const handleSubmit = (e: FormEvent, loginValue: string, passwordValue: string) => {
    e.preventDefault();
    login({ login: loginValue, password: passwordValue });
  };

  if (isPending || isSuccess)
    return (
      <div className='flex items-center justify-center h-screen w-screen bg-background'>
        <motion.img
          src={isDarkTheme ? scyllaLogoDark : scyllaLogo}
          alt='Scylla'
          className='h-40 w-72'
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: [0, 1, 1, 0.5], scale: [0.8, 1, 1, 0.95] }}
          transition={{ duration: 2, repeat: Infinity, ease: 'easeInOut' }}
        />
      </div>
    );

  return (
    <div className={'flex items-center flex-col'}>
      <ScyllaLogo className='w-2/6 h-2/6' />
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
