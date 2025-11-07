import { UserInformation } from '@/modules/user_settings/presentation/ui/UserInformation.tsx';
import { Organizations } from '@/modules/user_settings/presentation/ui/Organizations.tsx';

export const UserSettingsPage = () => {
  return (
    <div className='flex flex-col h-full'>
      <div className='w-full px-6 py-4 border-b flex items-center bg-[#f5f5f5]'>
        <select className='mr-2 text-lg font-medium text-gray-700'>
          <option value=''>Select an organization</option>
          <option value='org-001'>Alpha Organization</option>
          <option value='org-002'>Beta Organization</option>
          <option value='org-003'>Gamma Organization</option>
          <option value='org-004'>Delta Organization</option>
        </select>
      </div>

      <div className='flex pr-2 pl-2 space-x-6 bg-[#fafafa]'>
        <div className='w-1/2'>
          <UserInformation />
        </div>

        <div className='w-1/2'>
          <Organizations />
        </div>
      </div>
    </div>
  );
};

export default UserSettingsPage;
