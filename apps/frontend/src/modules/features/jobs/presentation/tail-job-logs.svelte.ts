import { getModuleDomain } from '@platform/di';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { JobsModule } from '../jobs.module.ts';

/** Lines are buffered and flushed on a timer: publishing per line is O(n²) and makes the stream lag and drop lines. */
const FLUSH_INTERVAL_MS = 150;

export interface TailedLogs {
  readonly text: string;
  readonly isLoading: boolean;
  readonly isError: boolean;
  readonly error: ScyllaError | null;
}

export interface TailJobLogsOptions {
  jobId: () => string;
  nodeId?: () => string | undefined;
}

/**
 * A job's logs as one ordered stream: the full history, then the live lines.
 * `jobId` and `nodeId` are getters: a panel is reused across jobs.
 */
export const createTailJobLogs = ({ jobId, nodeId }: TailJobLogsOptions): TailedLogs => {
  let text = $state('');
  let isLoading = $state(true);
  let isError = $state(false);
  let error = $state<ScyllaError | null>(null);

  $effect(() => {
    const currentJobId = jobId();
    const currentNodeId = nodeId?.();
    if (!currentJobId) return;

    let active = true;
    const lines: string[] = [];
    let dirty = false;

    text = '';
    isLoading = true;
    isError = false;
    error = null;

    const stream = getModuleDomain<typeof JobsModule.domain>('jobs')
      .jobsRepository.tailLogs(currentJobId, currentNodeId)
      .fold({
        onSuccess: value => value,
        onError: failure => {
          isError = true;
          error = failure;
          isLoading = false;
          return null;
        },
      });

    if (!stream) return;

    const flush = () => {
      if (dirty && active) {
        dirty = false;
        text = lines.join('\n');
      }
    };
    const flushTimer = window.setInterval(flush, FLUSH_INTERVAL_MS);

    const consume = async () => {
      isLoading = false;
      try {
        for await (const entry of stream.logs) {
          if (!active) break;
          entry.fold({
            onSuccess: log => {
              lines.push(log.line);
              dirty = true;
            },
            onError: failure => failure.log(),
          });
        }
      } catch {
        // The stream was cancelled on cleanup.
      }
      flush(); // final flush, so the last lines aren't lost between ticks
    };

    void consume();

    return () => {
      active = false;
      window.clearInterval(flushTimer);
      stream.cancel();
    };
  });

  return {
    get text() {
      return text;
    },
    get isLoading() {
      return isLoading;
    },
    get isError() {
      return isError;
    },
    get error() {
      return error;
    },
  };
};
