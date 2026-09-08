import type { ReactNode } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { UserInformation } from '@/modules/features/user/presentation/ui/settings/UserInformation.tsx';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';

interface UserSettingsPageProps {
  /**
   * The user's organizations panel, injected by whoever owns the route. Listing
   * organizations is the organization module's job, and it already depends on
   * `user` — so it fills this slot rather than being imported from here.
   */
  organizations?: ReactNode;
}

//TODO: change and list only organization that the user is in
export const UserSettingsPage = ({ organizations }: UserSettingsPageProps) => {
  const { userId } = useParams();

  return (
    <div className='flex flex-col gap-4 w-full'>
      <div className='flex items-center gap-4'>
        <h1 className='text-3xl font-bold'>
          <Trans>User settings</Trans>
        </h1>
      </div>

      <div className='flex space-x-6 bg-background'>
        <div className='w-1/2'>
          <UserInformation userId={userId ?? localStorage.getItem('userId') ?? undefined} />
        </div>

        {organizations && (
          <div className='w-1/2'>
            <Card className='w-full'>
              <CardHeader>
                <CardTitle>Organizations: </CardTitle>
              </CardHeader>
              <CardContent className='space-y-4'>{organizations}</CardContent>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
};

export default UserSettingsPage;
