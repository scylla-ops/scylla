import { type CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListUsersResponse, UserResponse, UpdateUserRequest } from '@/generated/user.ts';
import { UserServiceClient } from '@/generated/user.client.ts';
import type { UserRemoteDataSource } from '@/modules/features/user/infrastructure/repository/data-sources/user-remote.data-source.ts';
import { GrantServiceClient } from '@/generated/permission.client.ts';
import { Scope } from '@/generated/permission.ts';
import { wrapId } from '@core/infrastructure/grpc/wrappers.ts';

export class UserRemoteDataSourceImpl implements UserRemoteDataSource {
  private readonly _userClient: UserServiceClient;
  private readonly _grantClient: GrantServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._userClient = new UserServiceClient(transport.getTransport());
    this._grantClient = new GrantServiceClient(transport.getTransport());
  }

  public async getAll(): Promise<ScyllaResult<ListUsersResponse>> {
    return ScyllaResult.tryAsync<ListUsersResponse>(
      async () => await this._userClient.listUsers({}).response,
      'Failed to fetch users.',
    );
  }

  public async getById(userId: string): Promise<ScyllaResult<UserResponse>> {
    return ScyllaResult.tryAsync<UserResponse>(
      async () => await this._userClient.getUser({ userId: wrapId(userId) }).response,
      'Error fetching user',
    );
  }

  public async create(username: string, password: string): Promise<ScyllaResult<UserResponse>> {
    return ScyllaResult.tryAsync<UserResponse>(async () => {
      const user = await this._userClient.createUser({ username, password }).response;

      // Temporary: grant full access to the new user via a System-scoped
      // `system-admin` grant until the permissions system is finalized (a grant
      // on the System root confers control over the whole tenancy tree).
      await this._grantClient.createGrant({
        userId: user.userId,
        role: 'system-admin',
        scope: Scope.SYSTEM,
        scopeId: '',
      }).response;

      return user;
    }, 'Failed to create user.');
  }

  public async update(request: UpdateUserRequest): Promise<ScyllaResult<UserResponse>> {
    return ScyllaResult.tryAsync<UserResponse>(
      async () => await this._userClient.updateUser(request).response,
      'Failed to update user.',
    );
  }

  public async delete(userId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._userClient.deleteUser({ userId: wrapId(userId) }).response;
    }, 'Failed to delete user.');
  }
}
