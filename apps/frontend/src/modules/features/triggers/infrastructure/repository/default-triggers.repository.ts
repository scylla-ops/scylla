import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type {
  CreatedTrigger,
  TriggerDraft,
  TriggerEntity,
} from '@/modules/features/triggers/domain/entities/trigger.entity.ts';
import type { TriggersRepository } from '@/modules/features/triggers/domain/repository/triggers.repository.ts';
import type { TriggersRemoteDataSource } from '@/modules/features/triggers/infrastructure/repository/data-sources/triggers-remote.data-source.ts';
import { GrpcTriggerMapper } from '@/modules/features/triggers/infrastructure/repository/mappers/grpc-trigger.mapper.ts';
import type { Job } from '@/modules/features/jobs/domain/models/job.model.ts';
import { GrpcJobMapper } from '@/modules/features/jobs/infrastructure/repository/mappers/grpc-job.mapper.ts';

/** TriggersRepository backed by the remote (gRPC) data source. */
export class DefaultTriggersRepository implements TriggersRepository {
  constructor(private readonly remoteDataSource: TriggersRemoteDataSource) {}

  public async listByPipelineId(pipelineId: string): Promise<ScyllaResult<TriggerEntity[]>> {
    return (await this.remoteDataSource.listByPipelineId(pipelineId)).map(
      GrpcTriggerMapper.toDomainList,
    );
  }

  public async getById(triggerId: string): Promise<ScyllaResult<TriggerEntity>> {
    return (await this.remoteDataSource.getById(triggerId)).map(GrpcTriggerMapper.toDomain);
  }

  public async create(
    pipelineId: string,
    draft: TriggerDraft,
  ): Promise<ScyllaResult<CreatedTrigger>> {
    const request = GrpcTriggerMapper.draftToCreateRequest(pipelineId, draft);
    return (await this.remoteDataSource.create(request)).map(GrpcTriggerMapper.createdToDomain);
  }

  public async update(
    triggerId: string,
    draft: TriggerDraft,
  ): Promise<ScyllaResult<TriggerEntity>> {
    const request = GrpcTriggerMapper.draftToUpdateRequest(triggerId, draft);
    return (await this.remoteDataSource.update(request)).map(GrpcTriggerMapper.toDomain);
  }

  public deleteById(triggerId: string): Promise<ScyllaResult<boolean>> {
    return this.remoteDataSource.deleteById(triggerId);
  }

  public async setEnabled(
    triggerId: string,
    enabled: boolean,
  ): Promise<ScyllaResult<TriggerEntity>> {
    return (await this.remoteDataSource.setEnabled(triggerId, enabled)).map(
      GrpcTriggerMapper.toDomain,
    );
  }

  public async fireNow(triggerId: string): Promise<ScyllaResult<Job>> {
    return (await this.remoteDataSource.fireNow(triggerId)).map(GrpcJobMapper.toDomain);
  }
}
