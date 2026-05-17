import type {
  Job,
  JobNodeExecution,
  JobLog,
  JobLogStream,
} from '@/modules/features/jobs/domain/models/job.model.ts';
import type {
  JobResponse,
  JobNodeResponse,
  JobLogEntry,
  ListJobsResponse,
  ListJobLogsResponse,
} from '@/generated/job.ts';
import type { PaginationInfo } from '@shared/domain/models/pagination.model.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { JobLogsTailHandleRepo } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';

export class GrpcJobMapper {
  private static nodeExecutionToDomain(node: JobNodeResponse): JobNodeExecution {
    return {
      id: node.nodeId,
      state: node.state,
      startedAt: node.startedAt,
      finishedAt: node.finishedAt,
    };
  }

  static toDomain(job: JobResponse): Job {
    return {
      id: job.jobId,
      pipelineId: job.pipelineId,
      status: job.status,
      nodeExecutions: job.nodeExecutions.map(GrpcJobMapper.nodeExecutionToDomain),
      createdAt: job.createdAt,
      updatedAt: job.updatedAt,
    };
  }

  static toDomainList(response: ListJobsResponse): PaginatedList<Job> {
    return {
      items: response.jobs.map(GrpcJobMapper.toDomain),
      pagination: response.pagination as PaginationInfo,
    };
  }

  static logEntryToDomain(entry: JobLogEntry): JobLog {
    return {
      id: entry.id,
      jobId: entry.jobId,
      nodeId: entry.nodeId,
      stream: entry.stream,
      line: entry.line,
      timestamp: entry.timestamp,
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
