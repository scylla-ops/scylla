import { CoreModule } from '@core/di/core.module.ts';
import type { TriggersRemoteDataSource } from '@/modules/features/triggers/infrastructure/repository/data-sources/triggers-remote.data-source.ts';
import { GrpcTriggersRemoteDataSource } from '@/modules/features/triggers/infrastructure/data/remote/grpc-triggers-remote.data-source.ts';
import { DefaultTriggersRepository } from '@/modules/features/triggers/infrastructure/repository/default-triggers.repository.ts';
import { ListPipelineTriggersUseCase } from '@/modules/features/triggers/domain/usecases/list-pipeline-triggers.use-case.ts';
import { GetTriggerUseCase } from '@/modules/features/triggers/domain/usecases/get-trigger.use-case.ts';
import { CreateTriggerUseCase } from '@/modules/features/triggers/domain/usecases/create-trigger.use-case.ts';
import { UpdateTriggerUseCase } from '@/modules/features/triggers/domain/usecases/update-trigger.use-case.ts';
import { DeleteTriggerUseCase } from '@/modules/features/triggers/domain/usecases/delete-trigger.use-case.ts';
import { SetTriggerEnabledUseCase } from '@/modules/features/triggers/domain/usecases/set-trigger-enabled.use-case.ts';
import { FireTriggerNowUseCase } from '@/modules/features/triggers/domain/usecases/fire-trigger-now.use-case.ts';

const triggersRemoteDataSource: TriggersRemoteDataSource = new GrpcTriggersRemoteDataSource(
  CoreModule.data.grpcTransport,
);
const triggersRepository = new DefaultTriggersRepository(triggersRemoteDataSource);

const listPipelineTriggers = new ListPipelineTriggersUseCase(triggersRepository);
const getTrigger = new GetTriggerUseCase(triggersRepository);
const createTrigger = new CreateTriggerUseCase(triggersRepository);
const updateTrigger = new UpdateTriggerUseCase(triggersRepository);
const deleteTrigger = new DeleteTriggerUseCase(triggersRepository);
const setTriggerEnabled = new SetTriggerEnabledUseCase(triggersRepository);
const fireTriggerNow = new FireTriggerNowUseCase(triggersRepository);

export const TriggersModule = {
  domain: {
    listPipelineTriggers,
    getTrigger,
    createTrigger,
    updateTrigger,
    deleteTrigger,
    setTriggerEnabled,
    fireTriggerNow,
  },
};
