import { OrganizationList } from '@/modules/features/organization/presentation/ui/OrganizationList.tsx';
import { div } from 'framer-motion/m';
import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { UserInformation } from '@/modules/features/user/presentation/ui/settings/UserInformation.tsx';
import { useParams } from 'react-router-dom';
import { Trans } from '@lingui/react/macro';

//TODO: change and list only organization that the user is in
// (pass it as a props from Organization module)
export const UserSettingsPage = () => {
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

        <div className='w-1/2'>
          <Card className='w-full'>
            <CardHeader>
              <CardTitle>Organizations: </CardTitle>
            </CardHeader>
            <CardContent className='space-y-4'>
              <OrganizationList Wrapper={div} />
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default UserSettingsPage;
