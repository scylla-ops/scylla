import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import type {
  JobNodeExecution,
  JobLog,
  JobLogStream,
} from '@/modules/features/jobs/domain/structs/job.struct.ts';
import type {
  JobResponse,
  JobNodeResponse,
  JobLogEntry,
  ListJobsResponse,
  ListJobLogsResponse,
} from '@/generated/job.ts';
import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { JobLogsTailHandleRepo } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import {
  idValue,
  timestampToIso,
  timestampToIsoOpt,
} from '@shared/infrastructure/grpc/wrappers.ts';

export class GrpcJobMapper {
  private static nodeExecutionToDomain(node: JobNodeResponse): JobNodeExecution {
    return {
      id: idValue(node.nodeId),
      state: node.state,
      startedAt: timestampToIsoOpt(node.startedAt),
      finishedAt: timestampToIsoOpt(node.finishedAt),
    };
  }

  static toDomain(job: JobResponse): JobEntity {
    return {
      id: idValue(job.jobId),
      pipelineId: idValue(job.pipelineId),
      status: job.status,
      nodeExecutions: job.nodeExecutions.map(GrpcJobMapper.nodeExecutionToDomain),
      createdAt: timestampToIso(job.createdAt),
      updatedAt: timestampToIso(job.updatedAt),
      startedAt: timestampToIsoOpt(job.startedAt),
      finishedAt: timestampToIsoOpt(job.finishedAt),
    };
  }

  static toDomainList(response: ListJobsResponse): PaginatedList<JobEntity> {
    return {
      items: response.jobs.map(GrpcJobMapper.toDomain),
      pagination: response.pagination as PaginationInfo,
    };
  }

  static logEntryToDomain(entry: JobLogEntry): JobLog {
    return {
      id: idValue(entry.id),
      jobId: idValue(entry.jobId),
      nodeId: idValue(entry.nodeId),
      stream: entry.stream,
      line: entry.line,
      timestamp: timestampToIso(entry.timestamp),
    };
  }

  static logsToDomainList(response: ListJobLogsResponse): PaginatedList<JobLog> {
    return {
      items: response.logs.map(GrpcJobMapper.logEntryToDomain),
      pagination: response.pagination as PaginationInfo,
    };
  }

  static logStreamToDomain(stream: JobLogsTailHandleRepo): JobLogStream {
    return {
      logs: {
        async *[Symbol.asyncIterator]() {
          for await (const response of stream.responses) {
            yield response.map(GrpcJobMapper.logEntryToDomain);
          }
        },
      },
      cancel: stream.cancel,
    };
  }
}
