import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface LoginRepository {
  //remote
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
