import {
  Avatar,
  AvatarFallback,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@shadcn';

import {
  type FormItem,
  FormItemType,
  type FormChange,
} from '@shared/presentation/structs/scylla-form.struct.ts';
import { ScyllaForm } from '@shared/presentation/ui/forms/ScyllaForm.tsx';
import { Trans } from '@lingui/react/macro';
import { useUser } from '@/modules/features/user/presentation/hooks/use-user.ts';
import { useUpdateUser } from '@/modules/features/user/presentation/hooks/use-update-user.ts';

interface UserInformationProps {
  userId?: string;
}

export const UserInformation = ({ userId }: UserInformationProps) => {
  const { user, isLoading, isError } = useUser(userId || undefined);
  const updateUserMutation = useUpdateUser();

  const handleSubmit = (values: FormChange[]) => {
    if (userId) {
      const formData = values.reduce<Record<string, string>>(
        (acc, { id, value }) => ({ ...acc, [id]: value }),
        {},
      );
      updateUserMutation.mutate({
        userId,
        username: formData.username,
      });
    }
  };

  if (userId == undefined) {
    return (
      <Card className='w-full bg-card'>
        <CardHeader>
          <CardTitle>
            <Trans>User information</Trans>
          </CardTitle>
          <CardDescription>
            <Trans>Manage your account details.</Trans>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className='text-center text-muted-foreground'>
            <Trans>User information not available</Trans>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (isLoading) {
    return (
      <Card className='w-full bg-card'>
        <CardHeader>
          <CardTitle>
            <Trans>User information</Trans>
          </CardTitle>
          <CardDescription>
            <Trans>Manage your account details.</Trans>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className='text-center text-muted-foreground'>
            <Trans>Loading user information...</Trans>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (isError || !user) {
    return (
      <Card className='w-full bg-card'>
        <CardHeader>
          <CardTitle>
            <Trans>User information</Trans>
          </CardTitle>
          <CardDescription>
            <Trans>Manage your account details.</Trans>
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className='text-center text-destructive'>
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
      disabled: false,
      defaultValue: user.username,
    },
    {
      label: 'User ID',
      placeholder: user.userId,
      id: 'user-id',
      type: FormItemType.Input,
      inputType: 'text',
      disabled: true,
      defaultValue: user.userId,
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
    <Card className='w-full bg-card'>
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
            <AvatarFallback>{getInitials(user.username)}</AvatarFallback>
          </Avatar>
          <div>
            <div className='text-base font-medium'>{user.username}</div>
            <div className='text-sm text-muted-foreground'>
              <Trans>Active account</Trans>
            </div>
          </div>
        </div>

        <ScyllaForm
          onSubmit={handleSubmit}
          items={FormItems}
          buttonLabel={'Save'}
          className={'gap-2'}
          isPending={updateUserMutation.isPending}
        />
      </CardContent>
    </Card>
  );
};
