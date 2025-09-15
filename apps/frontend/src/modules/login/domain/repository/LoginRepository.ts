import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';

export interface LoginRepository {
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
