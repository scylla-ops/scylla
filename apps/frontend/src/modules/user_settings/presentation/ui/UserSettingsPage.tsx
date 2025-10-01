import { LeftPanel } from '@/modules/user_settings/presentation/ui/LeftPanel.tsx';
import { UserInformation } from '@/modules/user_settings/presentation/ui/UserInformation.tsx';
import { Organizations } from '@/modules/user_settings/presentation/ui/Organizations.tsx';

export const UserSettingsPage = () => {

    return (
        <div className="flex flex-col h-screen">
            {/* Top bar */}
            <div className="w-full px-6 py-4 border-b bg-white flex items-center">
                <select className="mr-2 text-sm font-medium text-gray-700">
                    <option value="">Select an organization</option>
                    <option value="org-001">Alpha Organization</option>
                    <option value="org-002">Beta Organization</option>
                    <option value="org-003">Gamma Organization</option>
                    <option value="org-004">Delta Organization</option>
                </select>
            </div>

            <div className="flex flex-1">

                {/* Left sidebar */}
                <div className="w-64 border-r">
                    <LeftPanel />
                </div>

                {/* Main content */}
                <div className="flex-1 p-6 space-x-6 flex">

                    {/* User information */}
                    <div className="w-1/2">
                        <UserInformation />
                    </div>

                    {/* Organizations */}
                    <div className="w-1/2">
                        <Organizations />
                    </div>

                </div>
            </div>
        </div>
    );
};

export default UserSettingsPage;
