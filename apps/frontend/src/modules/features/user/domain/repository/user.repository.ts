import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import type { UserList } from '@/modules/features/user/domain/structs/user.struct.ts';

export interface UserRepository {
  getAll(): Promise<ScyllaResult<UserList>>;
  getById(id: string): Promise<ScyllaResult<UserEntity>>;
  create(username: string, password: string): Promise<ScyllaResult<UserEntity>>;
  update(userId: string, username?: string): Promise<ScyllaResult<UserEntity>>;
  delete(userId: string): Promise<ScyllaResult<void>>;
}
