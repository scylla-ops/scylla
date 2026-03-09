import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';

export interface LoginRemoteStore {
  login(username: string, password: string): Promise<ScyllaResult<void>>;
}
