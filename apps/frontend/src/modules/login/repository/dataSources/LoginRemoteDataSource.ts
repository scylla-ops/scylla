import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface LoginRemoteDataSource {
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
