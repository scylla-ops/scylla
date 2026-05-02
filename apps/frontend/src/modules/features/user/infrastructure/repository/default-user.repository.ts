import type { UserRemoteDataSource } from '@/modules/features/user/infrastructure/repository/data-sources/user-remote.data-source.ts';
import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';
import { GrpcUserMapper } from '@/modules/features/user/infrastructure/repository/mappers/grpc-user.mapper.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';

export class DefaultUserRepository implements UserRepository {
  constructor(private readonly _remoteDataSource: UserRemoteDataSource) {}

  public async getAll(): Promise<ScyllaResult<PaginatedList<User>>> {
    return (await this._remoteDataSource.getAll()).map(list => GrpcUserMapper.toDomainList(list));
  }

  public async getById(id: string): Promise<ScyllaResult<User>> {
    return (await this._remoteDataSource.getById(id)).map(user => GrpcUserMapper.toDomain(user));
  }

  public async create(username: string, password: string): Promise<ScyllaResult<User>> {
    return (await this._remoteDataSource.create(username, password)).map(user =>
      GrpcUserMapper.toDomain(user),
    );
  }

  public async delete(userId: string): Promise<ScyllaResult<void>> {
    return this._remoteDataSource.delete(userId);
  }
}
