import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useEffect, useState } from 'react';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';

/**
 * Subscribe to a job's logs as a single ordered stream: the backend replays the
 * full persisted history (untruncated) and then appends live lines, so the view
 * is complete regardless of when it is opened and stays live while the job runs.
 * Pass `nodeId` to scope it to one node's logs.
 *
 * Lines are buffered and flushed to state on a timer rather than per-line: a
 * noisy job emits thousands of lines in a burst, and a setState-per-line would
 * both rebuild the whole string each time (O(n^2)) and stall the stream reader
 * enough to lag the server-side broadcast (dropping lines). Buffering keeps the
 * reader fast so nothing is dropped.
 */
export const useTailJobLogs = (jobId: string, nodeId?: string) => {
  const { tailJobLogs } = useDependencies().jobs;

  const [logString, setLogString] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [isError, setIsError] = useState(false);
  const [error, setError] = useState<ScyllaError | null>(null);

  useEffect(() => {
    if (!jobId) return;

    let active = true;
    const lines: string[] = [];
    let dirty = false;

    setLogString('');
    setIsLoading(true);
    setIsError(false);
    setError(null);

    const stream = tailJobLogs.execute(jobId, nodeId).fold({
      onSuccess: value => value,
      onError: err => {
        setIsError(true);
        setError(err);
        setIsLoading(false);
        return null;
      },
    });

    if (!stream) return;

    const flush = () => {
      if (dirty && active) {
        dirty = false;
        setLogString(lines.join('\n'));
      }
    };
    const flushTimer = setInterval(flush, 150);

    const consume = async () => {
      setIsLoading(false);
      try {
        for await (const entry of stream.logs) {
          if (!active) break;
          entry.fold({
            onSuccess: log => {
              lines.push(log.line);
              dirty = true;
            },
            onError: err => err.log(),
          });
        }
      } catch {
        // Stream cancelled — expected on cleanup.
      }
      flush(); // final flush so the last lines aren't lost between ticks
    };

    void consume();

    return () => {
      active = false;
      clearInterval(flushTimer);
      stream.cancel();
    };
  }, [jobId, nodeId, tailJobLogs]);

  return { logString, isLoading, isError, error };
};
