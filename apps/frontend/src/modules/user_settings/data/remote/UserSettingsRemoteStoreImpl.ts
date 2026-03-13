import type { UserSettingsRemoteStore } from '@/modules/user_settings/data/remote/UserSettingsRemoteStore.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import type { OrganizationUser } from '@/modules/user_settings/domain/models/OrganizationUser.ts';
import { OrganizationServiceClient } from '@/generated/organization.client.ts';
import type { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';

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
    try {
      const { response } = await this._organizationClient.listOrganizationUsers({
        organizationId,
        pagination: {
          page,
          pageSize,
        },
      });

      const users: OrganizationUser[] = (response.users ?? []).map((user) => ({
        user_id: user.userId,
        username: user.username,
        role: user.role,
        joined_at: user.joinedAt,
      }));

      return { ok: true, value: { users, pagination: response.pagination } };
    } catch (error) {
      return { ok: false, error: { message: `Failed to fetch organization users: ${error}` } };
    }
  }

  async addUserToOrganization(
    userId: string,
    organizationId: string,
    role: string,
  ): Promise<ScyllaResult<void>> {
    try {
      await this._organizationClient.addUserToOrganization({
        userId,
        organizationId,
        role,
      });
      return { ok: true, value: undefined };
    } catch (error) {
      return { ok: false, error: { message: `Failed to add user to organization: ${error}` } };
    }
  }

  async removeUserFromOrganization(
    userId: string,
    organizationId: string,
  ): Promise<ScyllaResult<void>> {
    try {
      await this._organizationClient.removeUserFromOrganization({
        userId,
        organizationId,
      });
      return { ok: true, value: undefined };
    } catch (error) {
      return {
        ok: false,
        error: { message: `Failed to remove user from organization: ${error}` },
      };
    }
  }
}
