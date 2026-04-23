import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@shadcn';

import { type FormItem, FormItemType } from '@core/presentation/models/ScyllaForm.ts';
import { ScyllaForm } from '@shared/presentation/ui/ScyllaForm.tsx';
import { useUser } from '@/modules/features/user_settings/presentation/hooks/useUserSettings.ts';
import { Trans } from '@lingui/react/macro';

export const UserInformation = () => {
  const userId = localStorage.getItem('userId');
  const { user, isLoading, isError } = useUser(userId || undefined);

  if (!userId) {
    return (
      <Card className='w-full bg-white'>
        <CardHeader>
          <CardTitle>
            <Trans>User information</Trans>
          </CardTitle>
          <CardDescription>
            <Trans>Manage your account details.</Trans>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className='text-center text-gray-500'>
            <Trans>User information not available</Trans>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (isLoading) {
    return (
      <Card className='w-full bg-white'>
        <CardHeader>
          <CardTitle>
            <Trans>User information</Trans>
          </CardTitle>
          <CardDescription>
            <Trans>Manage your account details.</Trans>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className='text-center text-gray-500'>
            <Trans>Loading user information...</Trans>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (isError || !user) {
    return (
      <Card className='w-full bg-white'>
        <CardHeader>
          <CardTitle>
            <Trans>User information</Trans>
          </CardTitle>
          <CardDescription>
            <Trans>Manage your account details.</Trans>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className='text-center text-red-500'>
            <Trans>Error loading user information</Trans>
          </div>
        </CardContent>
      </Card>
    );
  }

  const FormItems: FormItem[] = [
    {
      label: 'Username',
      placeholder: user.username,
      id: 'username',
      type: FormItemType.Input,
      inputType: 'text',
      disabled: true,
    },
    {
      label: 'User ID',
      placeholder: user.user_id,
      id: 'user-id',
      type: FormItemType.Input,
      inputType: 'text',
      disabled: true,
    },
  ];

  const getInitials = (username: string) => {
    return username
      .split(' ')
      .map(n => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  return (
    <Card className='w-full bg-white'>
      <CardHeader>
        <CardTitle>
          <Trans>User information</Trans>
        </CardTitle>
        <CardDescription>
          <Trans>Manage your account details.</Trans>
        </CardDescription>
      </CardHeader>

      <CardContent className='space-y-4'>
        <div className='flex items-center space-x-4'>
          <Avatar>
            <AvatarImage src={`https://github.com/${user.username}.png`} />
            <AvatarFallback>{getInitials(user.username)}</AvatarFallback>
          </Avatar>
          <div>
            <div className='text-base font-medium'>{user.username}</div>
            <div className='text-sm text-gray-500'>
              <Trans>Active account</Trans>
            </div>
          </div>
        </div>

        <ScyllaForm
          onSubmit={v => console.log(v)}
          items={FormItems}
          buttonLabel={'Save'}
          className={'gap-2'}
        />
      </CardContent>
    </Card>
  );
};
