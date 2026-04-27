import type { PipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import { GrpcPipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/data/remote/grpc-pipeline-remote.data-source.ts';
import { DefaultPipelineRepository } from '@/modules/features/pipeline/infrastructure/repository/default-pipeline.repository.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { GetPipelinesUseCase } from '@/modules/features/pipeline/domain/usecases/get-pipelines.use-case.ts';
import { DeletePipelineUseCase } from '@/modules/features/pipeline/domain/usecases/delete-pipeline.use-case.ts';
import { RunPipelinesUseCase } from '@/modules/features/pipeline/domain/usecases/run-pipelines.use-case.ts';
import { CreatePipelineUseCase } from '@/modules/features/pipeline/domain/usecases/create-pipeline.use-case.ts';

const pipelineRemoteDataSource: PipelineRemoteDataSource = new GrpcPipelineRemoteDataSource(
  CoreModule.data.grpcTransport,
);
const pipelineRepository = new DefaultPipelineRepository(pipelineRemoteDataSource);

const getPipelines = new GetPipelinesUseCase(pipelineRepository);
const deletePipeline = new DeletePipelineUseCase(pipelineRepository);
const runPipeline = new RunPipelinesUseCase(pipelineRepository);
const createPipeline = new CreatePipelineUseCase(pipelineRepository);

export const PipelineModule = {
  domain: { getPipelines, deletePipeline, runPipeline, createPipeline },
};
