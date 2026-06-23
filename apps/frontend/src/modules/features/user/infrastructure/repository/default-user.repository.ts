import type { UserRemoteDataSource } from '@/modules/features/user/infrastructure/repository/data-sources/user-remote.data-source.ts';
import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import { GrpcUserMapper } from '@/modules/features/user/infrastructure/repository/mappers/grpc-user.mapper.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';

export class DefaultUserRepository implements UserRepository {
  constructor(private readonly _remoteDataSource: UserRemoteDataSource) {}

  public async getAll(): Promise<ScyllaResult<PaginatedList<UserEntity>>> {
    return (await this._remoteDataSource.getAll()).map(list => GrpcUserMapper.toDomainList(list));
  }

  public async getById(id: string): Promise<ScyllaResult<UserEntity>> {
    return (await this._remoteDataSource.getById(id)).map(user => GrpcUserMapper.toDomain(user));
  }

  public async create(username: string, password: string): Promise<ScyllaResult<UserEntity>> {
    return (await this._remoteDataSource.create(username, password)).map(user =>
      GrpcUserMapper.toDomain(user),
    );
  }

  public async update(userId: string, username?: string): Promise<ScyllaResult<UserEntity>> {
    return (await this._remoteDataSource.update({ userId: wrapId(userId), username })).map(user =>
      GrpcUserMapper.toDomain(user),
    );
  }

  public async delete(userId: string): Promise<ScyllaResult<void>> {
    return this._remoteDataSource.delete(userId);
  }
}
