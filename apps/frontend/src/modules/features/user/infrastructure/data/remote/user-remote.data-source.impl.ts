import { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListUsersResponse, UserResponse, UpdateUserRequest } from '@/generated/user.ts';
import { UserServiceClient } from '@/generated/user.client.ts';
import type { UserRemoteDataSource } from '@/modules/features/user/infrastructure/repository/data-sources/user-remote.data-source.ts';
import { RoleServiceClient } from '@/generated/permission.client.ts';

export class UserRemoteDataSourceImpl implements UserRemoteDataSource {
  private readonly _userClient: UserServiceClient;
  private readonly _roleClient: RoleServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._userClient = new UserServiceClient(transport.getTransport());
    this._roleClient = new RoleServiceClient(transport.getTransport());
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

      // Temporary: grant full access to the new user via the admin role until the
      // permissions system is finalized (Cedar's admin policy grants all access).
      await this._roleClient.assignRole({
        userId: user.userId,
        role: 'admin',
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
      await this._userClient.deleteUser({ userId }).response;
    }, 'Failed to delete user.');
  }
}
