import type { UserSettingsRepository } from '@/modules/user_settings/domain/repository/UserSettingsRepository.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/user_settings/domain/models/OrganizationUser.ts';
import type { UserSettingsRemoteStore } from '@/modules/user_settings/data/remote/UserSettingsRemoteStore.ts';

export class UserSettingsRepositoryImpl implements UserSettingsRepository {
  constructor(private readonly remoteStore: UserSettingsRemoteStore) {}

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
    // Remove user and add back with new role
    const removeResult = await this.removeUserFromOrganization(userId, organizationId);
    if (!removeResult.ok) {
      return removeResult;
    }
    return this.addUserToOrganization(userId, organizationId, newRole);
  }
}
