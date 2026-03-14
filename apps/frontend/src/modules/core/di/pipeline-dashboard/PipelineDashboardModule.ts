import type { PipelineDashboardRemoteDataSource } from '@/modules/features/pipeline-dashboard/infrastructure/repository/data-sources/PipelineDashboardRemoteDataSource.ts';
import { PipelineDashboardRemoteDataSourceImpl } from '@/modules/features/pipeline-dashboard/infrastructure/data/remote/PipelineDashboardRemoteDataSourceImpl.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';
import { GetPipelinesUseCase } from '@/modules/features/pipeline-dashboard/domain/usecases/GetPipelinesUseCase.ts';
import { PipelineDashboardRepositoryImpl } from '@/modules/features/pipeline-dashboard/infrastructure/repository/PipelineDashboardRepositoryImpl.ts';

const pipelineDashboardRemoteDataSource: PipelineDashboardRemoteDataSource =
  new PipelineDashboardRemoteDataSourceImpl(CoreModule.data.coreGrpcTransport);
const pipelineDashboardRepository = new PipelineDashboardRepositoryImpl(
  pipelineDashboardRemoteDataSource,
);

const getPipelinesUseCase = new GetPipelinesUseCase(pipelineDashboardRepository);

export const PipelineDashboardModule = {
  domain: { getPipelinesUseCase: getPipelinesUseCase },
};
