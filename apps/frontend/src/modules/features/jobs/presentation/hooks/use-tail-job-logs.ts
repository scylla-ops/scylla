import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useEffect, useRef, useState } from 'react';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { JobLogStream } from '@/modules/features/jobs/domain/models/job.model.ts';

export const useTailJobLogs = (jobId: string, nodeId?: string) => {
  const { tailJobLogs } = useDependencies().jobs;

  const [logString, setLogString] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [isError, setIsError] = useState(false);
  const [error, setError] = useState<ScyllaError | null>(null);
  const streamRef = useRef<JobLogStream | null>(null);
  const activeJobIdRef = useRef<string | null>(null);

  useEffect(() => {
    if (!jobId) return;

    if (activeJobIdRef.current === jobId) return;

    if (streamRef.current) {
      streamRef.current.cancel();
      streamRef.current = null;
    }

    activeJobIdRef.current = jobId;
    setLogString('');
    setIsLoading(true);
    setIsError(false);
    setError(null);

    const result = tailJobLogs.execute(jobId, nodeId);

    const stream = result.fold({
      onSuccess: value => value,
      onError: err => {
        setIsError(true);
        setError(err);
        setIsLoading(false);
        activeJobIdRef.current = null;
        return null;
      },
    });

    if (!stream) return;

    streamRef.current = stream;

    const consumeStream = async () => {
      setIsLoading(false);
      try {
        for await (const entry of stream.logs) {
          if (streamRef.current !== stream) break;
          entry.fold({
            onSuccess: log => {
              setLogString(prev => (prev ? prev + '\n' + log.line : log.line));
            },
            onError: err => {
              err.log();
            },
          });
        }
      } catch {
        // Stream cancelled
      }
    };

    consumeStream();
  }, [jobId, tailJobLogs]);

  useEffect(() => {
    return () => {
      if (streamRef.current) {
        streamRef.current.cancel();
        streamRef.current = null;
        activeJobIdRef.current = null;
      }
    };
  }, []);

  return { logString, isLoading, isError, error };
};
