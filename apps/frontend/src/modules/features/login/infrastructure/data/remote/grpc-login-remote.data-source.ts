import type { LoginRemoteDataSource } from '@/modules/features/login/infrastructure/repository/data-sources/login-remote.data-source.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import { AuthServiceClient } from '@/generated/scylla/auth/v1/auth.client.ts';
import type { ScyllaGrpcTransport } from '@platform/grpc';
import { idValue } from '@shared/infrastructure/grpc/wrappers.ts';
import { t } from '@lingui/core/macro';

/**
 * gRPC answers `UNAUTHENTICATED` both for an expired session and for wrong
 * credentials. Only this call can mean the second one, and the global mutation
 * handler in `core`'s `App.tsx` reads the first: it clears the token and does a
 * full page load to `/login` — which, from the login page, silently reloads it
 * and swallows the error. Re-code it here so the ambiguity never leaves the
 * transport boundary. The original cause is dropped on purpose: the backend
 * message can hint at whether the account exists.
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

      //TODO: http cokkies instead of that
      localStorage.setItem('token', response.token);
      localStorage.setItem('userId', idValue(response.userId));
    }, 'Failed to login.');

    return result.mapError(error =>
      error.getCode() === 'UNAUTHENTICATED' ? invalidCredentials() : error,
    );
  }
}
