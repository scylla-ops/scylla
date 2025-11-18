import { UserInformation } from '@/modules/user_settings/presentation/ui/UserInformation.tsx';
import { Organizations } from '@/modules/user_settings/presentation/ui/Organizations.tsx';

export const UserSettingsPage = () => {
  return (
    <div className='flex pr-2 pl-2 space-x-6 bg-background'>
      <div className='w-1/2'>
        <UserInformation />
      </div>

      <div className='w-1/2'>
        <Organizations />
      </div>
    </div>
  );
};

export default UserSettingsPage;
