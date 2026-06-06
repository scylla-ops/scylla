import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useEffect, useState } from 'react';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';

/**
 * Subscribe to a job's logs as a single ordered stream: the backend replays the
 * full persisted history (untruncated) and then appends live lines, so the view
 * is complete regardless of when it is opened and stays live while the job runs.
 * Pass `nodeId` to scope it to one node's logs.
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

    const consume = async () => {
      setIsLoading(false);
      try {
        for await (const entry of stream.logs) {
          if (!active) break;
          entry.fold({
            onSuccess: log => setLogString(prev => (prev ? prev + '\n' + log.line : log.line)),
            onError: err => err.log(),
          });
        }
      } catch {
        // Stream cancelled — expected on cleanup.
      }
    };

    consume();

    return () => {
      active = false;
      stream.cancel();
    };
  }, [jobId, nodeId, tailJobLogs]);

  return { logString, isLoading, isError, error };
};
