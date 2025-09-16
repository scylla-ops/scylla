import type { LoginStore } from '@/modules/login/repository/store/LoginStore.ts';
import type { ScyllaResult } from '@/modules/core/domain/ScyllaResult.ts';
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';
import { AuthServiceClient } from '@/generated/auth.client.ts';

export class LoginStoreImpl implements LoginStore {
  private readonly _authClient: AuthServiceClient;

  constructor() {
    const apiUrl = import.meta.env.VITE_API_URL ?? '';
    const transport = new GrpcWebFetchTransport({
      baseUrl: apiUrl,
      format: 'binary',
    });
    this._authClient = new AuthServiceClient(transport);
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
