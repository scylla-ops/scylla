import { ConfigServiceClient } from '@/generated/config.client.ts';
import type { ServerConfigResponse } from '@/generated/config.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';

/// Reads the server's compiled-in feature set (cargo features) so the UI can
/// show/hide SaaS-only features. Public endpoint — no auth required.
export class ServerConfigDataSource {
  private readonly _client: ConfigServiceClient;

  constructor(transport: CoreGrpcTransport) {
    this._client = new ConfigServiceClient(transport.getTransport());
  }

  public async getServerConfig(): Promise<ServerConfigResponse> {
    const { response } = await this._client.getServerConfig({});
    return response;
  }
}
