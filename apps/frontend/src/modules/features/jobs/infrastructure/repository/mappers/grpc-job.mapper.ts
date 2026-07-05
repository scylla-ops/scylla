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
import { JobOutcome, NodeOutcome } from '@/generated/job.ts';
import type { PaginationInfo } from '@shared/domain/structs/pagination.struct.ts';
import type { PaginatedList } from '@shared/domain/types/paginated-list.type.ts';
import type { JobLogsTailHandleRepo } from '@/modules/features/jobs/infrastructure/repository/data-sources/jobs-remote.data-source.ts';
import {
  idValue,
  timestampToIso,
  timestampToIsoOpt,
} from '@shared/infrastructure/grpc/wrappers.ts';

/** Flatten the `JobResponse.state` oneof back to the flat status string the UI
 * consumes (`pending` | `running` | `completed` | `failed` | `cancelled` |
 * `orphaned`). An absent state is a malformed response — surface it as pending. */
function jobStatusFromState(state: JobResponse['state']): string {
  switch (state.oneofKind) {
    case 'pending':
      return 'pending';
    case 'running':
      return 'running';
    case 'terminal':
      return jobOutcomeToStatus(state.terminal.outcome);
    default:
      return 'pending';
  }
}

function jobOutcomeToStatus(outcome: JobOutcome): string {
  switch (outcome) {
    case JobOutcome.JOB_COMPLETED:
      return 'completed';
    case JobOutcome.JOB_FAILED:
      return 'failed';
    case JobOutcome.JOB_CANCELLED:
      return 'cancelled';
    case JobOutcome.JOB_ORPHANED:
      return 'orphaned';
    default:
      return 'completed';
  }
}

/** Flatten the `JobNodeResponse.execution` oneof to the flat state string the UI
 * consumes (`pending` | `running` | `completed` | `failed` | `cancelled` |
 * `skipped`). */
function nodeStateFromExecution(execution: JobNodeResponse['execution']): string {
  switch (execution.oneofKind) {
    case 'pending':
      return 'pending';
    case 'running':
      return 'running';
    case 'finished':
      return nodeOutcomeToState(execution.finished.outcome);
    default:
      return 'pending';
  }
}

function nodeOutcomeToState(outcome: NodeOutcome): string {
  switch (outcome) {
    case NodeOutcome.NODE_COMPLETED:
      return 'completed';
    case NodeOutcome.NODE_FAILED:
      return 'failed';
    case NodeOutcome.NODE_CANCELLED:
      return 'cancelled';
    case NodeOutcome.NODE_SKIPPED:
      return 'skipped';
    default:
      return 'completed';
  }
}

/** When execution actually began, projected from whichever state carries a start. */
function jobStartedAt(state: JobResponse['state']): string | undefined {
  if (state.oneofKind === 'running') return timestampToIsoOpt(state.running.startedAt);
  if (state.oneofKind === 'terminal') return timestampToIsoOpt(state.terminal.startedAt);
  return undefined;
}

function nodeStartedAt(execution: JobNodeResponse['execution']): string | undefined {
  if (execution.oneofKind === 'running') return timestampToIsoOpt(execution.running.startedAt);
  if (execution.oneofKind === 'finished') return timestampToIsoOpt(execution.finished.startedAt);
  return undefined;
}

export class GrpcJobMapper {
  private static nodeExecutionToDomain(node: JobNodeResponse): JobNodeExecution {
    const finished = node.execution.oneofKind === 'finished' ? node.execution.finished : undefined;
    return {
      id: idValue(node.nodeId),
      state: nodeStateFromExecution(node.execution),
      startedAt: nodeStartedAt(node.execution),
      finishedAt: timestampToIsoOpt(finished?.finishedAt),
    };
  }

  static toDomain(job: JobResponse): JobEntity {
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
