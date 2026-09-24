import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreateTriggerRequest,
  CreateTriggerResponse,
  Trigger,
  UpdateTriggerRequest,
} from '@/generated/scylla/trigger/v1/trigger.ts';

export interface TriggersRemoteDataSource {
  listByPipelineId(pipelineId: string): Promise<ScyllaResult<Trigger[]>>;
  getById(triggerId: string): Promise<ScyllaResult<Trigger>>;
  create(request: CreateTriggerRequest): Promise<ScyllaResult<CreateTriggerResponse>>;
  update(request: UpdateTriggerRequest): Promise<ScyllaResult<Trigger>>;
  deleteById(triggerId: string): Promise<ScyllaResult<void>>;
  setEnabled(triggerId: string, enabled: boolean): Promise<ScyllaResult<Trigger>>;
  fireNow(triggerId: string): Promise<ScyllaResult<string>>;
}
