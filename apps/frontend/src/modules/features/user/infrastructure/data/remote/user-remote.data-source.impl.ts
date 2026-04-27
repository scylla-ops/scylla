import { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListUsersResponse, UserResponse } from '@/generated/user.ts';
import { UserServiceClient } from '@/generated/user.client.ts';
import type { UserRemoteDataSource } from '@/modules/features/user/infrastructure/repository/data-sources/user-remote.data-source.ts';
import { PermissionServiceClient } from '@/generated/permission.client.ts';
import { Act, ResourceType, ScopeType } from '@/generated/permission.ts';

export class UserRemoteDataSourceImpl implements UserRemoteDataSource {
  private readonly _userClient: UserServiceClient;
  private readonly _permissionsClient: PermissionServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._userClient = new UserServiceClient(transport.getTransport());
    this._permissionsClient = new PermissionServiceClient(transport.getTransport());
  }

  public async getAll(): Promise<ScyllaResult<ListUsersResponse>> {
    return ScyllaResult.tryAsync<ListUsersResponse>(
      async () => await this._userClient.listUsers({}).response,
      'Failed to fetch users.',
    );
  }

  public async getById(userId: string): Promise<ScyllaResult<UserResponse>> {
    return ScyllaResult.tryAsync<UserResponse>(
      async () => await this._userClient.getUser({ userId }).response,
      'Error fetching user',
    );
  }

  public async create(username: string, password: string): Promise<ScyllaResult<UserResponse>> {
    return ScyllaResult.tryAsync<UserResponse>(async () => {
      const user = await this._userClient.createUser({ username, password }).response;

      // Temporary: grant all permissions to the new user until permissions system is finalized
      await this._permissionsClient.addPolicy({
        subject: user.userId,
        scope: { type: ScopeType.SCOPE_ALL },
        resource: { type: ResourceType.RESOURCE_ALL },
        act: Act.ALL,
      }).response;

      return user;
    }, 'Failed to create user.');
  }

  public async delete(userId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._userClient.deleteUser({ userId }).response;
    }, 'Failed to delete user.');
  }
}
