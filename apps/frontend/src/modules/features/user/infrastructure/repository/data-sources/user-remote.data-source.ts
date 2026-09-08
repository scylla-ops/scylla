import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListUsersResponse, User, UpdateUserRequest } from '@/generated/scylla/user/v1/user.ts';

export interface UserRemoteDataSource {
  getAll(): Promise<ScyllaResult<ListUsersResponse>>;
  getById(id: string): Promise<ScyllaResult<User>>;
  create(username: string, password: string): Promise<ScyllaResult<User>>;
  update(request: UpdateUserRequest): Promise<ScyllaResult<User>>;
  delete(userId: string): Promise<ScyllaResult<void>>;
}
