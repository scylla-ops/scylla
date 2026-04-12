import type { UserSettingsRemoteStore } from '@/modules/features/user_settings/infrastructure/repository/store/UserSettingsRemoteStore.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/features/user_settings/domain/models/OrganizationUser.ts';
import { OrganizationServiceClient } from '@/generated/organization.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';

export class UserSettingsRemoteStoreImpl implements UserSettingsRemoteStore {
  private readonly _organizationClient: OrganizationServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._organizationClient = new OrganizationServiceClient(transport.getTransport());
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
