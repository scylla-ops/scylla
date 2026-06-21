import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedTrigger,
  CreateTriggerRequest,
  ListTriggersResponse,
  TriggerView,
  UpdateTriggerRequest,
} from '@/generated/trigger.ts';
import type { JobResponse } from '@/generated/job.ts';

/**
 * Transport contract for the trigger service. Returns proto types — the
 * repository maps them to domain via {@link GrpcTriggerMapper}.
 */
export interface TriggersRemoteDataSource {
  listByPipelineId(pipelineId: string): Promise<ScyllaResult<ListTriggersResponse>>;
  getById(triggerId: string): Promise<ScyllaResult<TriggerView>>;
  create(request: CreateTriggerRequest): Promise<ScyllaResult<CreatedTrigger>>;
  update(request: UpdateTriggerRequest): Promise<ScyllaResult<TriggerView>>;
  deleteById(triggerId: string): Promise<ScyllaResult<boolean>>;
  setEnabled(triggerId: string, enabled: boolean): Promise<ScyllaResult<TriggerView>>;
  fireNow(triggerId: string): Promise<ScyllaResult<JobResponse>>;
}
