import type { ScyllaResult } from '@shared/utils/scylla-result.ts';

export interface JobNodeExecution {
  id: string;
  state: string;
  startedAt?: string;
  finishedAt?: string;
}

export interface Job {
  id: string;
  pipelineId: string;
  status: string;
  nodeExecutions: JobNodeExecution[];
  createdAt: string;
  updatedAt: string;
  /** When execution actually began (a worker picked it up). Unset while pending. */
  startedAt?: string;
  /** When execution finished. Unset while pending/running. */
  finishedAt?: string;
}

export interface JobLog {
  id: string;
  jobId: string;
  nodeId: string;
  stream: string;
  line: string;
  timestamp: string;
}

export interface JobLogStream {
  logs: AsyncIterable<ScyllaResult<JobLog>>;
  cancel: () => void;
}
