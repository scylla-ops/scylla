import type { JobsRepository } from '@/modules/features/jobs/domain/repository/jobs.repository.ts';
import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import type { JobLog, JobLogStream } from '@/modules/features/jobs/domain/structs/job.struct.ts';
import type { ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { JobsRemoteDataSource } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import type { PaginationParams } from '@shared/domain/structs/pagination.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import { GrpcJobMapper } from '@/modules/features/jobs/infrastructure/repository/mappers/grpc-job.mapper.ts';

export class DefaultJobsRepository implements JobsRepository {
  constructor(private readonly remoteDataSource: JobsRemoteDataSource) {}

  public async getByPipelineId(
    pipelineId: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<JobEntity>>> {
    return (await this.remoteDataSource.getByPipelineId(pipelineId, pagination)).map(
      GrpcJobMapper.toDomainList,
    );
  }

  public async getById(jobId: string): Promise<ScyllaResult<JobEntity>> {
    return (await this.remoteDataSource.getById(jobId)).map(GrpcJobMapper.toDomain);
  }

  public async deleteById(jobId: string): Promise<ScyllaResult<void>> {
    return this.remoteDataSource.deleteById(jobId);
  }

  public async getLogs(
    jobId: string,
    nodeId?: string,
    pagination?: PaginationParams,
  ): Promise<ScyllaResult<PaginatedList<JobLog>>> {
    return (await this.remoteDataSource.getLogs(jobId, nodeId, pagination)).map(
      GrpcJobMapper.logsToDomainList,
    );
  }

  public tailLogs(jobId: string, nodeId?: string): ScyllaResult<JobLogStream> {
    return this.remoteDataSource.tailLogs(jobId, nodeId).map(GrpcJobMapper.logStreamToDomain);
  }
}
