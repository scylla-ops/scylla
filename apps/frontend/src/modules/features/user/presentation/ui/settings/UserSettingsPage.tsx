import { OrganizationList } from '@/modules/features/organization/presentation/ui/OrganizationList.tsx';
import { div } from 'framer-motion/m';
import { Card, CardContent, CardHeader, CardTitle } from '@shadcn';
import { UserInformation } from '@/modules/features/user/presentation/ui/settings/UserInformation.tsx';

//TODO: change and list only organization that the user is in
// (pass it as a props from Organization module)
export const UserSettingsPage = () => {
  return (
    <div className='flex space-x-6 bg-background'>
      <div className='w-1/2'>
        <UserInformation />
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
  );
};

export default UserSettingsPage;
