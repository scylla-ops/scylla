import { type CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  ListUsersResponse,
  UpdateUserRequest,
  User,
} from '@/generated/scylla/user/v1/user.ts';
import { UserServiceClient } from '@/generated/scylla/user/v1/user.client.ts';
import type { UserRemoteDataSource } from '@/modules/features/user/infrastructure/repository/data-sources/user-remote.data-source.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';

/**
 * Every user RPC now answers with a `XxxResponse` wrapper holding the entity in
 * field 1. The entity is `optional` on the wire, so a response without it means
 * the server broke its own contract: fail loudly here rather than let an empty
 * user reach the mappers.
 */
function requireUser(user: User | undefined): User {
  if (!user) throw new Error('Server returned a response without a user.');
  return user;
}

export class UserRemoteDataSourceImpl implements UserRemoteDataSource {
  private readonly _userClient: UserServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._userClient = new UserServiceClient(transport.getTransport());
  }

  public async getAll(): Promise<ScyllaResult<ListUsersResponse>> {
    return ScyllaResult.tryAsync<ListUsersResponse>(
      async () => await this._userClient.listUsers({}).response,
      'Failed to fetch users.',
    );
  }

  public async getById(userId: string): Promise<ScyllaResult<User>> {
    return ScyllaResult.tryAsync<User>(
      async () =>
        requireUser((await this._userClient.getUser({ userId: wrapId(userId) }).response).user),
      'Error fetching user',
    );
  }

  public async create(username: string, password: string): Promise<ScyllaResult<User>> {
    return ScyllaResult.tryAsync<User>(async () => {
      return requireUser((await this._userClient.createUser({ username, password }).response).user);
    }, 'Failed to create user.');
  }

  public async update(request: UpdateUserRequest): Promise<ScyllaResult<User>> {
    return ScyllaResult.tryAsync<User>(
      async () => requireUser((await this._userClient.updateUser(request).response).user),
      'Failed to update user.',
    );
  }

  public async delete(userId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._userClient.deleteUser({ userId: wrapId(userId) }).response;
    }, 'Failed to delete user.');
  }
}
