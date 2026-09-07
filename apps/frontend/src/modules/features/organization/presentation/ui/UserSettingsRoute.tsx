import { div } from 'framer-motion/m';
import { UserSettingsPage } from '@/modules/features/user/presentation/ui/settings/UserSettings.page.tsx';
import { OrganizationList } from '@/modules/features/organization/presentation/ui/OrganizationList.tsx';

/**
 * User settings with the organizations panel filled in.
 *
 * The page belongs to `user`; listing organizations belongs here. Composing on
 * this side keeps the dependency one-way (organization → user) instead of the
 * mutual import the panel used to require.
 */
export const UserSettingsRoute = () => <UserSettingsPage organizations={<OrganizationList Wrapper={div} />} />;
