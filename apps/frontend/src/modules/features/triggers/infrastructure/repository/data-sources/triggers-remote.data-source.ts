import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreateTriggerRequest,
  CreateTriggerResponse,
  Trigger,
  UpdateTriggerRequest,
} from '@/generated/scylla/trigger/v1/trigger.ts';

/**
 * Transport contract for the trigger service. Unwraps the `XxxResponse`
 * envelopes and returns proto entities — the repository maps them to domain
 * via {@link GrpcTriggerMapper}.
 */
export interface TriggersRemoteDataSource {
  listByPipelineId(pipelineId: string): Promise<ScyllaResult<Trigger[]>>;
  getById(triggerId: string): Promise<ScyllaResult<Trigger>>;
  create(request: CreateTriggerRequest): Promise<ScyllaResult<CreateTriggerResponse>>;
  update(request: UpdateTriggerRequest): Promise<ScyllaResult<Trigger>>;
  deleteById(triggerId: string): Promise<ScyllaResult<void>>;
  setEnabled(triggerId: string, enabled: boolean): Promise<ScyllaResult<Trigger>>;
  /** Fires the trigger and returns the id of the job it minted. */
  fireNow(triggerId: string): Promise<ScyllaResult<string>>;
}
