import type { LoginRemoteStore } from '@/modules/login/repository/store/LoginRemoteStore.ts';
import { ScyllaResult } from '@core/utils/ScyllaResult.ts';
import { AuthServiceClient } from '@/generated/auth.client.ts';
import type { CoreGrpcTransport } from '@core/data/grpc/CoreGrpcTransport.ts';

export class LoginRemoteStoreImpl implements LoginRemoteStore {
  private readonly _authClient: AuthServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._authClient = new AuthServiceClient(transport.getTransport());
  }

  public async login(username: string, password: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      const { response } = await this._authClient.login({ username, password });
      localStorage.setItem('token', response.token);
    }, 'Failed to login.');
  }
}
