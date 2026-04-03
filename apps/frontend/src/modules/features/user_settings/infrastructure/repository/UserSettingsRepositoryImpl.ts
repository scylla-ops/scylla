import type { UserSettingsRepository } from '@/modules/features/user_settings/domain/repository/UserSettingsRepository.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/features/user_settings/domain/models/OrganizationUser.ts';
import type { User } from '@/modules/features/user_settings/domain/models/User.ts';
import type { UserSettingsRemoteStore } from '@/modules/features/user_settings/infrastructure/repository/store/UserSettingsRemoteStore.ts';

export class UserSettingsRepositoryImpl implements UserSettingsRepository {
  constructor(private readonly remoteStore: UserSettingsRemoteStore) {}

  async getUser(userId: string): Promise<ScyllaResult<User>> {
    return this.remoteStore.getUser(userId);
  }

  async getOrganizationUsers(
    organizationId: string,
    page?: number,
    pageSize?: number,
  ): Promise<ScyllaResult<{ users: OrganizationUser[]; pagination: any }>> {
    return this.remoteStore.getOrganizationUsers(organizationId, page, pageSize);
  }

  async addUserToOrganization(
    userId: string,
    organizationId: string,
    role: string,
  ): Promise<ScyllaResult<void>> {
    return this.remoteStore.addUserToOrganization(userId, organizationId, role);
  }

  async removeUserFromOrganization(
    userId: string,
    organizationId: string,
  ): Promise<ScyllaResult<void>> {
    return this.remoteStore.removeUserFromOrganization(userId, organizationId);
  }

  async updateUserRole(
    userId: string,
    organizationId: string,
    newRole: string,
  ): Promise<ScyllaResult<void>> {
    const res = await this.removeUserFromOrganization(userId, organizationId);

    return res.fold({
      onSuccess: () => this.addUserToOrganization(userId, organizationId, newRole),
      onError: _ => Promise.resolve(res),
    });
  }
}
