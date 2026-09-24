import type { JobEntity } from '@/modules/features/jobs/domain/entities/job.entity.ts';
import type {
  JobNodeExecution,
  JobLog,
  JobLogStream,
} from '@/modules/features/jobs/domain/structs/job.struct.ts';
import type {
  Job,
  JobNode,
  JobLogEntry,
  ListPipelineJobsResponse,
  ListJobLogsResponse,
} from '@/generated/scylla/job/v1/job.ts';
import { JobOutcome, NodeOutcome } from '@/generated/scylla/job/v1/job.ts';
import { LogStream } from '@/generated/scylla/common/v1/common.ts';
import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { JobLogsTailHandleRepo } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';

type JobListResponse = Pick<ListPipelineJobsResponse, 'jobs' | 'pagination'>;
import {
  idValue,
  timestampToIso,
  timestampToIsoOpt,
} from '@shared/infrastructure/grpc/wrappers.ts';

/** An arm newer than this build becomes `unknown`. */
function jobStatusFromState(state: Job['state']): string {
  switch (state.oneofKind) {
    case 'pending':
      return 'pending';
    case 'running':
      return 'running';
    case 'terminal':
      return jobOutcomeToStatus(state.terminal.outcome);
    default:
      return 'unknown';
  }
}

function jobOutcomeToStatus(outcome: JobOutcome): string {
  switch (outcome) {
    case JobOutcome.COMPLETED:
      return 'completed';
    case JobOutcome.FAILED:
      return 'failed';
    case JobOutcome.CANCELLED:
      return 'cancelled';
    case JobOutcome.ORPHANED:
      return 'orphaned';
    // UNSPECIFIED or a newer outcome: never a success.
    default:
      return 'unknown';
  }
}

/** An arm newer than this build becomes `unknown`. */
function nodeStateFromExecution(execution: JobNode['execution']): string {
  switch (execution.oneofKind) {
    case 'pending':
      return 'pending';
    case 'running':
      return 'running';
    case 'finished':
      return nodeOutcomeToState(execution.finished.outcome);
    default:
      return 'unknown';
  }
}

function nodeOutcomeToState(outcome: NodeOutcome): string {
  switch (outcome) {
    case NodeOutcome.COMPLETED:
      return 'completed';
    case NodeOutcome.FAILED:
      return 'failed';
    case NodeOutcome.CANCELLED:
      return 'cancelled';
    case NodeOutcome.SKIPPED:
      return 'skipped';
    // UNSPECIFIED or a newer outcome: never a success.
    default:
      return 'unknown';
  }
}

/** `UNSPECIFIED` or an unknown stream gives `''`. */
function logStreamToString(stream: LogStream): string {
  switch (stream) {
    case LogStream.STDOUT:
      return 'stdout';
    case LogStream.STDERR:
      return 'stderr';
    default:
      return '';
  }
}

function jobStartedAt(state: Job['state']): string | undefined {
  if (state.oneofKind === 'running') return timestampToIsoOpt(state.running.startedAt);
  if (state.oneofKind === 'terminal') return timestampToIsoOpt(state.terminal.startedAt);
  return undefined;
}

function nodeStartedAt(execution: JobNode['execution']): string | undefined {
  if (execution.oneofKind === 'running') return timestampToIsoOpt(execution.running.startedAt);
  if (execution.oneofKind === 'finished') return timestampToIsoOpt(execution.finished.startedAt);
  return undefined;
}

export class GrpcJobMapper {
  private static nodeExecutionToDomain(node: JobNode): JobNodeExecution {
    const finished = node.execution.oneofKind === 'finished' ? node.execution.finished : undefined;
    return {
      id: idValue(node.nodeId),
      state: nodeStateFromExecution(node.execution),
      startedAt: nodeStartedAt(node.execution),
      finishedAt: timestampToIsoOpt(finished?.finishedAt),
    };
  }

  static toDomain(job: Job): JobEntity {
    const terminal = job.state.oneofKind === 'terminal' ? job.state.terminal : undefined;
    return {
      id: idValue(job.jobId),
      pipelineId: idValue(job.pipelineId),
      status: jobStatusFromState(job.state),
      nodeExecutions: job.nodeExecutions.map(GrpcJobMapper.nodeExecutionToDomain),
      createdAt: timestampToIso(job.createdAt),
      updatedAt: timestampToIso(job.updatedAt),
      startedAt: jobStartedAt(job.state),
      finishedAt: timestampToIsoOpt(terminal?.finishedAt),
    };
  }

  /** The pipeline, project and organization listings share this shape. */
  static toDomainList(response: JobListResponse): PaginatedList<JobEntity> {
    return {
      items: response.jobs.map(GrpcJobMapper.toDomain),
      pagination: response.pagination as PaginationInfo,
    };
  }

  static logEntryToDomain(entry: JobLogEntry): JobLog {
    return {
      id: idValue(entry.jobLogId),
      jobId: idValue(entry.jobId),
      nodeId: idValue(entry.nodeId),
      stream: logStreamToString(entry.stream),
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
