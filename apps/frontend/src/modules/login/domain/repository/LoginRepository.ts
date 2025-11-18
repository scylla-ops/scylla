import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';

export interface LoginRepository {
  //remote
  login(username: string, password: string): Promise<ScyllaResult<string>>;
}
