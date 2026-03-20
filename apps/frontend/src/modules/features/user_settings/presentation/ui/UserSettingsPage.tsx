import { UserInformation } from '@/modules/features/user_settings/presentation/ui/UserInformation.tsx';
import { Organization } from '@/modules/features/user_settings/presentation/ui/Organization.tsx';

export const UserSettingsPage = () => {
  return (
    <div className='flex space-x-6 bg-background'>
      <div className='w-1/2'>
        <UserInformation />
      </div>

      <div className='w-1/2'>
        <Organization />
      </div>
    </div>
  );
};

export default UserSettingsPage;
