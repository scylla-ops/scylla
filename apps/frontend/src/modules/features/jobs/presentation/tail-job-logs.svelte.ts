import { getModuleDomain } from '@platform/di';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { JobsModule } from '../jobs.module.ts';

/**
 * How often buffered lines are handed to the view.
 *
 * Lines are buffered and flushed on a timer rather than published per line: a
 * noisy job emits thousands in a burst, and re-publishing the whole string per
 * line would both rebuild it each time (O(n²)) and stall the stream reader
 * enough to lag the server-side broadcast, which drops lines. Buffering keeps
 * the reader fast so nothing is lost.
 */
const FLUSH_INTERVAL_MS = 150;

export interface TailedLogs {
  /** Every line received so far, newline-joined. Only ever grows, or restarts. */
  readonly text: string;
  readonly isLoading: boolean;
  readonly isError: boolean;
  readonly error: ScyllaError | null;
}

export interface TailJobLogsOptions {
  jobId: () => string;
  /** Scopes the stream to one node's logs. */
  nodeId?: () => string | undefined;
}

/**
 * Subscribes to a job's logs as a single ordered stream: the backend replays the
 * full persisted history (untruncated) and then appends live lines, so the view
 * is complete regardless of when it is opened and stays live while the job runs.
 *
 * A server stream is a system outside the framework, which is what `$effect` is
 * for — one of the few places in this codebase where the React original's
 * `useEffect` was already the right tool. What changes is the honesty of the
 * dependencies: the effect re-runs when `jobId` or `nodeId` actually change,
 * with no array to keep in step, and the cleanup that cancels the stream is the
 * same one Svelte runs on unmount.
 *
 * `jobId` and `nodeId` are getters: a panel is reused across jobs, and reading
 * either once would pin the subscription to whichever job happened to be open
 * first.
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
        // Stream cancelled — expected on cleanup.
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
