import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';

export interface LoginStore {
  login(username: string, password: string): Promise<ScyllaResult<string>>;
}
