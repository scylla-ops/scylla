import type { UserSettingsRemoteStore } from '@/modules/user_settings/data/remote/UserSettingsRemoteStore.ts';
import { UserSettingsRemoteStoreImpl } from '@/modules/user_settings/data/remote/UserSettingsRemoteStoreImpl.ts';
import type { UserSettingsRepository } from '@/modules/user_settings/domain/repository/UserSettingsRepository.ts';
import { UserSettingsRepositoryImpl } from '@/modules/user_settings/repository/UserSettingsRepositoryImpl.ts';
import { GetOrganizationUsersUseCase } from '@/modules/user_settings/domain/usecases/GetOrganizationUsersUseCase.ts';
import { AddUserToOrganizationUseCase } from '@/modules/user_settings/domain/usecases/AddUserToOrganizationUseCase.ts';
import { RemoveUserFromOrganizationUseCase } from '@/modules/user_settings/domain/usecases/RemoveUserFromOrganizationUseCase.ts';
import { UpdateUserRoleUseCase } from '@/modules/user_settings/domain/usecases/UpdateUserRoleUseCase.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';

const userSettingsRemoteStore: UserSettingsRemoteStore = new UserSettingsRemoteStoreImpl(
  CoreModule.data.coreGrpcTransport,
);
const userSettingsRepository: UserSettingsRepository = new UserSettingsRepositoryImpl(
  userSettingsRemoteStore,
);

const getOrganizationUsersUseCase = new GetOrganizationUsersUseCase(userSettingsRepository);
const addUserToOrganizationUseCase = new AddUserToOrganizationUseCase(userSettingsRepository);
const removeUserFromOrganizationUseCase = new RemoveUserFromOrganizationUseCase(
  userSettingsRepository,
);
const updateUserRoleUseCase = new UpdateUserRoleUseCase(userSettingsRepository);

export const UserSettingsModule = {
  domain: {
    getOrganizationUsersUseCase,
    addUserToOrganizationUseCase,
    removeUserFromOrganizationUseCase,
    updateUserRoleUseCase,
  },
  repository: userSettingsRepository,
};
