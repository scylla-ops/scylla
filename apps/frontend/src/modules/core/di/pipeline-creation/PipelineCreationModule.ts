import { PipelineCreationRemoteDataSourceImpl } from '@/modules/features/pipeline-creation/infrastructure/data/remote/PipelineCreationRemoteDataSourceImpl.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';
import { PipelineCreationRepositoryImpl } from '@/modules/features/pipeline-creation/infrastructure/repository/PipelineCreationRepositoryImpl.ts';
import { CreatePipelineUsecase } from '@/modules/features/pipeline-creation/domain/CreatePipelineUsecase.ts';

const pipelineCreationRemoteDataSource = new PipelineCreationRemoteDataSourceImpl(
  CoreModule.data.coreGrpcTransport,
);
const pipelineCreationRepository = new PipelineCreationRepositoryImpl(
  pipelineCreationRemoteDataSource,
);

const createPipelineUseCase = new CreatePipelineUsecase(pipelineCreationRepository);

export const PipelineCreationModule = {
  domain: { createPipelineUseCase: createPipelineUseCase },
};
