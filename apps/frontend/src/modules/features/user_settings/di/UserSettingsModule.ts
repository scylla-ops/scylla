import type { UserSettingsRemoteStore } from '@/modules/features/user_settings/infrastructure/repository/store/UserSettingsRemoteStore.ts';
import { UserSettingsRemoteStoreImpl } from '@/modules/features/user_settings/infrastructure/data/remote/UserSettingsRemoteStoreImpl.ts';
import type { UserSettingsRepository } from '@/modules/features/user_settings/domain/repository/UserSettingsRepository.ts';
import { UserSettingsRepositoryImpl } from '@/modules/features/user_settings/infrastructure/repository/UserSettingsRepositoryImpl.ts';
import { GetOrganizationUsers } from '@/modules/features/user_settings/domain/usecases/GetOrganizationUsers.ts';
import { AddUserToOrganization } from '@/modules/features/user_settings/domain/usecases/AddUserToOrganization.ts';
import { RemoveUserFromOrganization } from '@/modules/features/user_settings/domain/usecases/RemoveUserFromOrganization.ts';
import { UpdateUserRole } from '@/modules/features/user_settings/domain/usecases/UpdateUserRole.ts';
import { CoreModule } from '@core/di/CoreModule.ts';

const userSettingsRemoteStore: UserSettingsRemoteStore = new UserSettingsRemoteStoreImpl(
  CoreModule.data.grpcTransport,
);
const userSettingsRepository: UserSettingsRepository = new UserSettingsRepositoryImpl(
  userSettingsRemoteStore,
);

const getOrganizationUsers = new GetOrganizationUsers(userSettingsRepository);
const addUserToOrganization = new AddUserToOrganization(userSettingsRepository);
const removeUserFromOrganization = new RemoveUserFromOrganization(userSettingsRepository);
const updateUserRole = new UpdateUserRole(userSettingsRepository);

export const UserSettingsModule = {
  domain: {
    getOrganizationUsers,
    addUserToOrganization,
    removeUserFromOrganization,
    updateUserRole,
  },
};
