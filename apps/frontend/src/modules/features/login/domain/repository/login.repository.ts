import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface LoginRepository {
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
