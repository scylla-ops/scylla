import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import { TriggerServiceClient } from '@/generated/scylla/trigger/v1/trigger.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { idValue, wrapId } from '@shared/infrastructure/grpc/wrappers.ts';
import type {
  CreateTriggerRequest,
  CreateTriggerResponse,
  Trigger,
  UpdateTriggerRequest,
} from '@/generated/scylla/trigger/v1/trigger.ts';
import type { TriggersRemoteDataSource } from '@/modules/features/triggers/infrastructure/repository/data-sources/triggers-remote.data-source.ts';

/** The trigger-carrying responses all use the same field, so one guard covers them. */
function requireTrigger(trigger: Trigger | undefined, rpc: string): Trigger {
  if (!trigger) {
    throw new ScyllaError(`${rpc} returned no trigger`);
  }
  return trigger;
}

export class GrpcTriggersRemoteDataSource implements TriggersRemoteDataSource {
  private readonly _client: TriggerServiceClient;

  public constructor(transport: CoreGrpcTransport) {
    this._client = new TriggerServiceClient(transport.getTransport());
  }

  public async listByPipelineId(pipelineId: string): Promise<ScyllaResult<Trigger[]>> {
    return ScyllaResult.tryAsync<Trigger[]>(
      async () =>
        (await this._client.listPipelineTriggers({ pipelineId: wrapId(pipelineId) })).response
          .triggers,
      'Error listing triggers',
    );
  }

  public async getById(triggerId: string): Promise<ScyllaResult<Trigger>> {
    return ScyllaResult.tryAsync<Trigger>(async () => {
      const { response } = await this._client.getTrigger({ triggerId: wrapId(triggerId) });
      return requireTrigger(response.trigger, 'GetTrigger');
    }, 'Error fetching trigger');
  }

  public async create(request: CreateTriggerRequest): Promise<ScyllaResult<CreateTriggerResponse>> {
    return ScyllaResult.tryAsync<CreateTriggerResponse>(
      async () => (await this._client.createTrigger(request)).response,
      'Failed to create trigger',
    );
  }

  public async update(request: UpdateTriggerRequest): Promise<ScyllaResult<Trigger>> {
    return ScyllaResult.tryAsync<Trigger>(async () => {
      const { response } = await this._client.updateTrigger(request);
      return requireTrigger(response.trigger, 'UpdateTrigger');
    }, 'Failed to update trigger');
  }

  public async deleteById(triggerId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._client.deleteTrigger({ triggerId: wrapId(triggerId) }).response;
    }, 'Error deleting trigger');
  }

  public async setEnabled(triggerId: string, enabled: boolean): Promise<ScyllaResult<Trigger>> {
    return ScyllaResult.tryAsync<Trigger>(async () => {
      const { response } = await this._client.setTriggerEnabled({
        triggerId: wrapId(triggerId),
        enabled,
      });
      return requireTrigger(response.trigger, 'SetTriggerEnabled');
    }, 'Failed to update trigger');
  }

  public async fireNow(triggerId: string): Promise<ScyllaResult<string>> {
    // FireTriggerNow no longer returns the job, only the id it minted.
    return ScyllaResult.tryAsync<string>(
      async () =>
        idValue(
          (await this._client.fireTriggerNow({ triggerId: wrapId(triggerId) })).response.jobId,
        ),
      'Failed to fire trigger',
    );
  }
}
