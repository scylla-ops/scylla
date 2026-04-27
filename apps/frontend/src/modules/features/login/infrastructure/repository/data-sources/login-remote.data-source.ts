import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface LoginRemoteDataSource {
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
