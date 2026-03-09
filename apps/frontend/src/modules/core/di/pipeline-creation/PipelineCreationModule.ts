import { PipelineCreationRemoteStoreImpl } from '@/modules/pipeline-creation/data/remote/PipelineCreationRemoteStoreImpl.ts';
import { CoreModule } from '@core/di/core/CoreModule.ts';
import { PipelineCreationRepositoryImpl } from '@/modules/pipeline-creation/repository/PipelineCreationRepositoryImpl.ts';
import { CreatePipelineUsecase } from '@/modules/pipeline-creation/domain/CreatePipelineUsecase.ts';

const pipelineCreationRemoteStore = new PipelineCreationRemoteStoreImpl(
  CoreModule.data.coreGrpcTransport,
);
const pipelineCreationRepository = new PipelineCreationRepositoryImpl(pipelineCreationRemoteStore);

const createPipelineUseCase = new CreatePipelineUsecase(pipelineCreationRepository);

export const PipelineCreationModule = {
  domain: { createPipelineUseCase: createPipelineUseCase },
};
