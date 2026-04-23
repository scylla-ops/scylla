import type { UserSettingsRemoteStore } from '@/modules/features/user_settings/infrastructure/repository/store/UserSettingsRemoteStore.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/features/user_settings/domain/models/OrganizationUser.ts';
import type { User } from '@/modules/features/user_settings/domain/models/User.ts';
import { OrganizationServiceClient } from '@/generated/organization.client.ts';
import { UserServiceClient } from '@/generated/user.client.ts';
import type { GrpcTransport } from '@core/infrastructure/grpc/GrpcTransport.ts';

export class UserSettingsRemoteStoreImpl implements UserSettingsRemoteStore {
  private readonly _organizationClient: OrganizationServiceClient;
  private readonly _userClient: UserServiceClient;

  constructor(transport: GrpcTransport) {
    this._organizationClient = new OrganizationServiceClient(transport.getTransport());
    this._userClient = new UserServiceClient(transport.getTransport());
  }

  async getUser(userId: string): Promise<ScyllaResult<User>> {
    return ScyllaResult.tryAsync(async () => {
      const response = (await this._userClient.getUser({ userId })).response;

      return {
        user_id: response.userId,
        username: response.username,
        is_active: response.isActive,
        created_at: response.createdAt,
        updated_at: response.updatedAt,
      };
    }, 'Error fetching user');
  }

  async getOrganizationUsers(
    organizationId: string,
    page: number = 1,
    pageSize: number = 10,
  ): Promise<ScyllaResult<{ users: OrganizationUser[]; pagination: any }>> {
    return ScyllaResult.tryAsync(async () => {
      const response = (
        await this._organizationClient.listOrganizationUsers({
          organizationId,
          pagination: {
            page,
            pageSize,
          },
        })
      ).response;

      const users: OrganizationUser[] = (response.users ?? []).map(user => ({
        user_id: user.userId,
        username: user.username,
        role: '',
        joined_at: '',
      }));

      return { users, pagination: response.pagination };
    }, 'Error listing organization users');
  }

  async addUserToOrganization(
    userId: string,
    organizationId: string,
    _: string,
  ): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._organizationClient.addUserToOrganization({
        userId,
        organizationId,
      });
    }, 'Error adding user to organization');
  }

  async removeUserFromOrganization(
    userId: string,
    organizationId: string,
  ): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync(async () => {
      await this._organizationClient.removeUserFromOrganization({
        userId,
        organizationId,
      });
    }, 'Error removing user from organization');
  }
}
