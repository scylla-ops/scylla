import type { PipelineDashboardRemoteDataSource } from '@/modules/pipeline-dashboard/repository/dataSources/PipelineDashboardRemoteDataSource.ts';
import { PipelineDashboardRemoteDataSourceImpl } from '@/modules/pipeline-dashboard/data/remote/PipelineDashboardRemoteDataSourceImpl.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';
import { GetPipelinesUseCase } from '@/modules/pipeline-dashboard/domain/usecases/GetPipelinesUseCase.ts';
import { PipelineDashboardRepositoryImpl } from '@/modules/pipeline-dashboard/repository/PipelineDashboardRepositoryImpl.ts';

const pipelineDashboardRemoteDataSource: PipelineDashboardRemoteDataSource =
  new PipelineDashboardRemoteDataSourceImpl(CoreModule.data.coreGrpcTransport);
const pipelineDashboardRepository = new PipelineDashboardRepositoryImpl(
  pipelineDashboardRemoteDataSource,
);

const getPipelinesUseCase = new GetPipelinesUseCase(pipelineDashboardRepository);

export const PipelineDashboardModule = {
  domain: { getPipelinesUseCase: getPipelinesUseCase },
};
