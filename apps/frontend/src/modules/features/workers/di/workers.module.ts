import { WorkersRemoteDataSourceImpl } from '@/modules/features/workers/infrastructure/data/workers-remote.data-source.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { DefaultWorkersRepository } from '@/modules/features/workers/infrastructure/repository/default-workers.repository.ts';
import { GetWorkersUseCase } from '@/modules/features/workers/domain/use-cases/get-workers.use-case.ts';
import { GetWorkerUseCase } from '@/modules/features/workers/domain/use-cases/get-worker.use-case.ts';
import { GetWorkerStatsUseCase } from '@/modules/features/workers/domain/use-cases/get-worker-stats.use-case.ts';
import { CreateWorkerUseCase } from '@/modules/features/workers/domain/use-cases/create-worker.use-case.ts';
import { DeleteWorkerUseCase } from '@/modules/features/workers/domain/use-cases/delete-worker.use-case.ts';

const dataSource = new WorkersRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const repository = new DefaultWorkersRepository(dataSource);

export const WorkersModule = {
  domain: {
    getWorkers: new GetWorkersUseCase(repository),
    getWorker: new GetWorkerUseCase(repository),
    getWorkerStats: new GetWorkerStatsUseCase(repository),
    createWorker: new CreateWorkerUseCase(repository),
    deleteWorker: new DeleteWorkerUseCase(repository),
  },
};
