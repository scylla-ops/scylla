import type { PipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/repository/data-sources/pipeline-remote.data-source.ts';
import { GrpcPipelineRemoteDataSource } from '@/modules/features/pipeline/infrastructure/data/remote/grpc-pipeline-remote.data-source.ts';
import { DefaultPipelineRepository } from '@/modules/features/pipeline/infrastructure/repository/default-pipeline.repository.ts';
import { CoreModule } from '@core/di/core.module.ts';
import { GetPipelinesMetadataUseCase } from '@/modules/features/pipeline/domain/usecases/get-pipelines-metadata.use-case.ts';
import { DeletePipelineUseCase } from '@/modules/features/pipeline/domain/usecases/delete-pipeline.use-case.ts';
import { RunPipelineUseCase } from '@/modules/features/pipeline/domain/usecases/run-pipeline.use-case.ts';
import { CreatePipelineUseCase } from '@/modules/features/pipeline/domain/usecases/create-pipeline.use-case.ts';
import { GetPipelineUseCase } from '@/modules/features/pipeline/domain/usecases/get-pipeline.use-case.ts';
import { UpdatePipelineUseCase } from '@/modules/features/pipeline/domain/usecases/update-pipeline.use-case.ts';

const pipelineRemoteDataSource: PipelineRemoteDataSource = new GrpcPipelineRemoteDataSource(
  CoreModule.data.grpcTransport,
);
const pipelineRepository = new DefaultPipelineRepository(pipelineRemoteDataSource);

const getPipelinesMetadata = new GetPipelinesMetadataUseCase(pipelineRepository);
const deletePipeline = new DeletePipelineUseCase(pipelineRepository);
const runPipeline = new RunPipelineUseCase(pipelineRepository);
const createPipeline = new CreatePipelineUseCase(pipelineRepository);
const getPipeline = new GetPipelineUseCase(pipelineRepository);
const updatePipeline = new UpdatePipelineUseCase(pipelineRepository);

export const PipelineModule = {
  domain: {
    getPipelinesMetadata,
    deletePipeline,
    runPipeline,
    createPipeline,
    getPipeline,
    updatePipeline,
  },
};
