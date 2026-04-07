import type { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';

export interface LoginRepository {
  //remote
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
