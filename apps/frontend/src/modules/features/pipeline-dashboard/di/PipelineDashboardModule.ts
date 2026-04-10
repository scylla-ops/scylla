import type { PipelineDashboardRemoteDataSource } from '@/modules/features/pipeline-dashboard/infrastructure/repository/data-sources/PipelineDashboardRemoteDataSource.ts';
import { PipelineDashboardRemoteDataSourceImpl } from '@/modules/features/pipeline-dashboard/infrastructure/data/remote/PipelineDashboardRemoteDataSourceImpl.ts';
import { GetPipelines } from '@/modules/features/pipeline-dashboard/domain/usecases/GetPipelines.ts';
import { PipelineDashboardRepositoryImpl } from '@/modules/features/pipeline-dashboard/infrastructure/repository/PipelineDashboardRepositoryImpl.ts';
import { CoreModule } from '@core/di/CoreModule.ts';
import { DeletePipeline } from '@/modules/features/pipeline-dashboard/domain/usecases/DeletePipeline.ts';
import { RunPipeline } from '../domain/usecases/RunPipeline';

const pipelineDashboardRemoteDataSource: PipelineDashboardRemoteDataSource =
  new PipelineDashboardRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const pipelineDashboardRepository = new PipelineDashboardRepositoryImpl(
  pipelineDashboardRemoteDataSource,
);

const getPipelines = new GetPipelines(pipelineDashboardRepository);
const deletePipeline = new DeletePipeline(pipelineDashboardRepository);
const runPipeline = new RunPipeline(pipelineDashboardRepository);

export const PipelineDashboardModule = {
  domain: { getPipelines, deletePipeline, runPipeline },
};
