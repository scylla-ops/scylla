import type { SecretRemoteDataSource } from '@/modules/features/secret/infrastructure/repository/data-sources/secret-remote.data-source.ts';
import { GrpcSecretRemoteDataSource } from '@/modules/features/secret/infrastructure/data/remote/grpc-secret-remote.data-source.ts';
import { DefaultSecretRepository } from '@/modules/features/secret/infrastructure/repository/default-secret.repository.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { ListSecretsUseCase } from '@/modules/features/secret/domain/usecases/list-secrets.use-case.ts';
import { CreateSecretUseCase } from '@/modules/features/secret/domain/usecases/create-secret.use-case.ts';
import { DeleteSecretUseCase } from '@/modules/features/secret/domain/usecases/delete-secret.use-case.ts';

const secretRemoteDataSource: SecretRemoteDataSource = new GrpcSecretRemoteDataSource(
  CoreModule.data.grpcTransport,
);
const secretRepository = new DefaultSecretRepository(secretRemoteDataSource);

const listSecrets = new ListSecretsUseCase(secretRepository);
const createSecret = new CreateSecretUseCase(secretRepository);
const deleteSecret = new DeleteSecretUseCase(secretRepository);

export const SecretModule = {
  domain: {
    listSecrets,
    createSecret,
    deleteSecret,
  },
};
