import type { PipelineDashboardRemoteDataSource } from '@/modules/pipeline-dashboard/repository/dataSources/PipelineDashboardRemoteDataSource.ts';
import { PipelineDashboardRemoteDataStoreImpl } from '@/modules/pipeline-dashboard/data/remote/PipelineDashboardRemoteDataStoreImpl.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';
import { GetPipelinesUseCase } from '@/modules/pipeline-dashboard/domain/usecases/GetPipelinesUseCase.ts';
import { PipelineDashboardRepositoryImpl } from '@/modules/pipeline-dashboard/repository/PipelineDashboardRepositoryImpl.ts';

const pipelineDashboardStore: PipelineDashboardRemoteDataSource =
  new PipelineDashboardRemoteDataStoreImpl(CoreModule.data.coreGrpcTransport);
const pipelineDashboardRepository = new PipelineDashboardRepositoryImpl(pipelineDashboardStore);

const getPipelinesUseCase = new GetPipelinesUseCase(pipelineDashboardRepository);

export const PipelineDashboardModule = {
  domain: { getPipelinesUseCase: getPipelinesUseCase },
};
