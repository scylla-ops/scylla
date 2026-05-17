import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { ListUsersResponse, UserResponse, UpdateUserRequest } from '@/generated/user.ts';

export interface UserRemoteDataSource {
  getAll(): Promise<ScyllaResult<ListUsersResponse>>;
  getById(id: string): Promise<ScyllaResult<UserResponse>>;
  create(username: string, password: string): Promise<ScyllaResult<UserResponse>>;
  update(request: UpdateUserRequest): Promise<ScyllaResult<UserResponse>>;
  delete(userId: string): Promise<ScyllaResult<void>>;
}
