import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface LoginRepository {
  //remote
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
