import { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';
import { ScyllaResult } from '@shared/utils/ScyllaResult.ts';
import type { ListUsersResponse, UserResponse } from '@/generated/user.ts';
import { UserServiceClient } from '@/generated/user.client.ts';
import type { UserRemoteDataSource } from '@/modules/features/user/infrastructure/repository/data-sources/user-remote.data-source.ts';

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

  public async getById(userId: string): Promise<ScyllaResult<UserResponse>> {
    return ScyllaResult.tryAsync<UserResponse>(
      async () => await this._userClient.getUser({ userId }).response,
      'Error fetching user',
    );
  }

  public async create(username: string, password: string): Promise<ScyllaResult<UserResponse>> {
    return ScyllaResult.tryAsync<UserResponse>(
      async () => await this._userClient.createUser({ username, password }).response,
      'Failed to create user.',
    );
  }
}
