import type { ScyllaResult } from '@shared/utils/ScyllaResult.ts';
import type { User, UserList } from '@/modules/features/user/domain/models/user.model.ts';

export interface UserRepository {
  getAll(): Promise<ScyllaResult<UserList>>;
  getById(id: string): Promise<ScyllaResult<User>>;
  create(username: string, password: string): Promise<ScyllaResult<User>>;
  delete(userId: string): Promise<ScyllaResult<void>>;
}
