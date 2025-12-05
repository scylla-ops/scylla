import type { PipelineDashboardStore } from '@/modules/pipeline-dashboard/repository/store/PipelineDashboardStore.ts';
import { PipelineDashboardStoreImpl } from '@/modules/pipeline-dashboard/data/remote/PipelineDashboardStoreImpl.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';
import { GetPipelinesUseCase } from '@/modules/pipeline-dashboard/domain/usecases/GetPipelinesUseCase.ts';
import { PipelineDashboardRepositoryImpl } from '@/modules/pipeline-dashboard/repository/PipelineDashboardRepositoryImpl.ts';

const pipelineDashboardStore: PipelineDashboardStore = new PipelineDashboardStoreImpl(
  CoreModule.data.coreGrpcTransport,
);
const pipelineDashboardRepository = new PipelineDashboardRepositoryImpl(pipelineDashboardStore);

const getPipelinesUseCase = new GetPipelinesUseCase(pipelineDashboardRepository);

export const PipelineDashboardModule = {
  domain: { getPipelinesUseCase: getPipelinesUseCase },
};
