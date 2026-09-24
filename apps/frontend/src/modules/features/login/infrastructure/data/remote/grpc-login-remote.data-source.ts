import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/login-remote.data-source.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import { AuthServiceClient } from '@/generated/scylla/auth/v1/auth.client.ts';
import type { ScyllaGrpcTransport } from '@platform/grpc';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import { t } from '@lingui/core/macro';

/**
 * UNAUTHENTICATED here means wrong credentials, not an expired session: re-code it,
 * or the global handler signs out and reloads the login page. The backend message
 * is dropped: it could tell whether the account exists.
 */
const invalidCredentials = () =>
  new ScyllaError(t`Incorrect username or password`, {
    cause: { code: 'INVALID_CREDENTIALS' },
  });

export class GrpcLoginRemoteDataSource implements LoginRemoteDataSource {
  private readonly _authClient: AuthServiceClient;

  constructor(transport: ScyllaGrpcTransport) {
    this._authClient = new AuthServiceClient(transport.getTransport());
  }

  public async login(identifier: string, password: string): Promise<ScyllaResult<void>> {
    const result = await ScyllaResult.tryAsync<void>(async () => {
      const { response } = await this._authClient.login({ identifier, password });

      // TODO: HTTP cookies instead.
      localStorage.setItem('token', response.token);
      localStorage.setItem('userId', idValue(response.userId));
    }, 'Failed to login.');

    return result.mapError(error =>
      error.getCode() === 'UNAUTHENTICATED' ? invalidCredentials() : error,
    );
  }
}
