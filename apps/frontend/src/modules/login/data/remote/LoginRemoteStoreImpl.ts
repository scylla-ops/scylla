import type { LoginRemoteStore } from '@/modules/login/repository/store/LoginRemoteStore.ts';
import type { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import { AuthServiceClient } from '@/generated/auth.client.ts';
import type { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';

export class LoginRemoteStoreImpl implements LoginRemoteStore {
  private readonly _authClient: AuthServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._authClient = new AuthServiceClient(transport.getTransport());
  }

  //TODO: add a logger (console error for developer, only if a debug env var is set) ??
  public async login(username: string, password: string): Promise<ScyllaResult<string>> {
    try {
      const { response } = await this._authClient.login({ username, password });
      return { ok: true, value: response.token };
    } catch (err) {
      console.error('Login failed:', err); //todo: here
      return { ok: false, error: { message: String(err) } };
    }
  }
}
