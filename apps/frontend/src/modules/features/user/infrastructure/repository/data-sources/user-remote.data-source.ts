import type { ScyllaResult } from '@shared/utils/ScyllaResult.ts';
import type { ListUsersResponse, UserResponse } from '@/generated/user.ts';

export interface UserRemoteDataSource {
  getAll(): Promise<ScyllaResult<ListUsersResponse>>;
  getById(id: string): Promise<ScyllaResult<UserResponse>>;
  create(username: string, password: string): Promise<ScyllaResult<UserResponse>>;
}
