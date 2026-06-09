import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { SecretServiceClient } from '@/generated/secret.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { wrapId } from '@core/infrastructure/grpc/wrappers.ts';
import type {
  CreateSecretInput,
  SecretEntity,
} from '@/modules/features/secret/domain/entities/secret.entity.ts';
import type { SecretRemoteDataSource } from '@/modules/features/secret/infrastructure/repository/data-sources/secret-remote.data-source.ts';
import { GrpcSecretMapper } from '@/modules/features/secret/infrastructure/repository/mappers/grpc-secret.mapper.ts';

export class GrpcSecretRemoteDataSource implements SecretRemoteDataSource {
  private readonly _secretClient: SecretServiceClient;

  public constructor(transport: CoreGrpcTransport) {
    this._secretClient = new SecretServiceClient(transport.getTransport());
  }

  public async listByProjectId(projectId: string): Promise<ScyllaResult<SecretEntity[]>> {
    return ScyllaResult.tryAsync<SecretEntity[]>(async () => {
      const response = await this._secretClient.listSecrets({
        projectId: wrapId(projectId),
      }).response;
      return response.secrets.map(GrpcSecretMapper.toDomain);
    }, 'Error listing secrets');
  }

  public async create(input: CreateSecretInput): Promise<ScyllaResult<SecretEntity>> {
    return ScyllaResult.tryAsync<SecretEntity>(async () => {
      const response = await this._secretClient.createSecret({
        projectId: wrapId(input.projectId),
        name: input.name,
        value: input.value,
        description: input.description,
      }).response;
      return GrpcSecretMapper.toDomain(response);
    }, 'Failed to create secret.');
  }

  public async deleteById(secretId: string): Promise<ScyllaResult<boolean>> {
    return ScyllaResult.tryAsync<boolean>(async () => {
      const response = await this._secretClient.deleteSecret({
        secretId: wrapId(secretId),
      }).response;
      return response.deleted;
    }, 'Error deleting secret');
  }
}
