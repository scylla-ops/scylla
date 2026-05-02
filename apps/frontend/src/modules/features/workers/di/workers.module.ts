import { WorkersRemoteDataSourceImpl } from '@/modules/features/workers/infrastructure/data/workers-remote.data-source.impl.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { DefaultWorkersRepository } from '@/modules/features/workers/infrastructure/repository/default-workers.repository.ts';
import { GetWorkersUseCase } from '@/modules/features/workers/domain/use-cases/get-workers.use-case.ts';
import { GetWorkerUseCase } from '@/modules/features/workers/domain/use-cases/get-worker.use-case.ts';

const dataSource = new WorkersRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const repository = new DefaultWorkersRepository(dataSource);

const getWorkersUseCase = new GetWorkersUseCase(repository);
const getWorkerUseCase = new GetWorkerUseCase(repository);

export const WorkersModule = {
  domain: {
    getWorkers: getWorkersUseCase,
    getWorker: getWorkerUseCase,
  },
};
