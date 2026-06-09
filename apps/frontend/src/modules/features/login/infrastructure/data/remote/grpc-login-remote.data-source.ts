import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/login-remote.data-source.ts';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { AuthServiceClient } from '@/generated/auth.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { idValue } from '@core/infrastructure/grpc/wrappers.ts';

export class GrpcLoginRemoteDataSource implements LoginRemoteDataSource {
  private readonly _authClient: AuthServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._authClient = new AuthServiceClient(transport.getTransport());
  }

  public async login(identifier: string, password: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      const { response } = await this._authClient.login({ identifier, password });

      //TODO: persist store zustand context ici
      localStorage.setItem('token', response.token);
      localStorage.setItem('userId', idValue(response.userId));
    }, 'Failed to login.');
  }
}
