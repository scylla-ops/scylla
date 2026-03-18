import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export interface LoginRemoteDataSource {
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
