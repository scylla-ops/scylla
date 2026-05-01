import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { useState } from 'react';

const WORKERS_QUERY_KEY = ['workers'];

export function useWorkers() {
  const { workers } = useDependencies();
  const [searchTerm, setSearchTerm] = useState('');

  const { data, isLoading, isError, error } = useQuery({
    queryKey: WORKERS_QUERY_KEY,
    queryFn: async () => {
      const result = await workers.getWorkers.execute();
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
  });

  // Filter workers based on search term
  const filtered = data?.workers?.filter(
    worker =>
      worker.hostname.toLowerCase().includes(searchTerm.toLowerCase()) ||
      worker.agentId.toLowerCase().includes(searchTerm.toLowerCase()) ||
      worker.status.toLowerCase().includes(searchTerm.toLowerCase()),
  );

  return {
    workers: filtered || [],
    allWorkers: data?.workers || [],
    isLoading,
    isError,
    error,
    searchTerm,
    setSearchTerm,
  };
}

export function useWorker(workerId: string) {
  const { workers } = useDependencies();

  return useQuery({
    queryKey: [...WORKERS_QUERY_KEY, workerId],
    queryFn: async () => {
      const result = await workers.getWorker.execute(workerId);
      return result.fold({
        onSuccess: data => data,
        onError: err => {
          throw err;
        },
      });
    },
    enabled: !!workerId,
  });
}
