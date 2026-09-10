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
import { ScyllaLoadingScreen } from '@shared/presentation/ui';

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

  const handleSubmit = (loginValue: string, passwordValue: string) => {
    login({ login: loginValue, password: passwordValue });
  };

  if (isSuccess) return <ScyllaLoadingScreen />;

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
          <LoginForm handleSubmit={handleSubmit} isPending={isPending} />
        </CardContent>
      </Card>
    </div>
  );
};

export default LoginPage;
