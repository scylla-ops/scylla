import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { TriggerServiceClient } from '@/generated/trigger.client.ts';
import type { CoreGrpcTransport } from '@core/infrastructure/grpc/core-grpc-transport.ts';
import { wrapId } from '@shared/infrastructure/grpc/wrappers.ts';
import type {
  CreatedTrigger,
  CreateTriggerRequest,
  ListTriggersResponse,
  TriggerView,
  UpdateTriggerRequest,
} from '@/generated/trigger.ts';
import type { JobResponse } from '@/generated/job.ts';
import type { TriggersRemoteDataSource } from '@/modules/features/triggers/infrastructure/repository/data-sources/triggers-remote.data-source.ts';

export class GrpcTriggersRemoteDataSource implements TriggersRemoteDataSource {
  private readonly _client: TriggerServiceClient;

  public constructor(transport: CoreGrpcTransport) {
    this._client = new TriggerServiceClient(transport.getTransport());
  }

  public async listByPipelineId(pipelineId: string): Promise<ScyllaResult<ListTriggersResponse>> {
    return ScyllaResult.tryAsync<ListTriggersResponse>(
      async () =>
        (await this._client.listPipelineTriggers({ pipelineId: wrapId(pipelineId) })).response,
      'Error listing triggers',
    );
  }

  public async getById(triggerId: string): Promise<ScyllaResult<TriggerView>> {
    return ScyllaResult.tryAsync<TriggerView>(
      async () => (await this._client.getTrigger({ triggerId: wrapId(triggerId) })).response,
      'Error fetching trigger',
    );
  }

  public async create(request: CreateTriggerRequest): Promise<ScyllaResult<CreatedTrigger>> {
    return ScyllaResult.tryAsync<CreatedTrigger>(
      async () => (await this._client.createTrigger(request)).response,
      'Failed to create trigger',
    );
  }

  public async update(request: UpdateTriggerRequest): Promise<ScyllaResult<TriggerView>> {
    return ScyllaResult.tryAsync<TriggerView>(
      async () => (await this._client.updateTrigger(request)).response,
      'Failed to update trigger',
    );
  }

  public async deleteById(triggerId: string): Promise<ScyllaResult<void>> {
    return ScyllaResult.tryAsync<void>(async () => {
      await this._client.deleteTrigger({ triggerId: wrapId(triggerId) }).response;
    }, 'Error deleting trigger');
  }

  public async setEnabled(triggerId: string, enabled: boolean): Promise<ScyllaResult<TriggerView>> {
    return ScyllaResult.tryAsync<TriggerView>(
      async () =>
        (await this._client.setTriggerEnabled({ triggerId: wrapId(triggerId), enabled })).response,
      'Failed to update trigger',
    );
  }

  public async fireNow(triggerId: string): Promise<ScyllaResult<JobResponse>> {
    return ScyllaResult.tryAsync<JobResponse>(
      async () => (await this._client.fireTriggerNow({ triggerId: wrapId(triggerId) })).response,
      'Failed to fire trigger',
    );
  }
}
