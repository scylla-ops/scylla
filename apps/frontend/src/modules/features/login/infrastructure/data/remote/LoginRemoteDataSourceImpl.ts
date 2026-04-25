import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/LoginRemoteDataSource.ts';
import { ScyllaResult } from '@/modules/shared/utils/ScyllaResult.ts';
import { AuthServiceClient } from '@/generated/auth.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/CoreGrpcTransport.ts';

export class LoginRemoteDataSourceImpl implements LoginRemoteDataSource {
  private readonly _authClient: AuthServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._authClient = new AuthServiceClient(transport.getTransport());
  }

  public async login(username: string, password: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      const { response } = await this._authClient.login({ username, password });

      //TODO: persist store zustand context ici
      localStorage.setItem('token', response.token);
      localStorage.setItem('userId', response.userId);
    }, 'Failed to login.');
  }
}
