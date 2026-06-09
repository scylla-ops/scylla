import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { Secret } from '@/modules/features/secret/domain/models/secret.model.ts';

const SECRETS_QUERY_KEY = 'secrets';

/**
 * Surface the most useful message from a failed secret call. The data source
 * wraps gRPC failures in a ScyllaError whose `cause` is the underlying RpcError —
 * that cause carries the real backend message (e.g. "secret store not
 * configured" or a duplicate-name conflict), so prefer it for the toast.
 */
function secretErrorMessage(error: unknown): string {
  if (error instanceof ScyllaError && error.cause instanceof Error) {
    return error.cause.message;
  }
  return error instanceof Error ? error.message : 'Une erreur est survenue';
}

/** List a project's secrets (metadata only — never a value). */
export const useSecrets = (projectId: string) => {
  const { listSecrets } = useDependencies().secret;

  const { data, isLoading, isError, error } = useQuery<Secret[], ScyllaError>({
    queryKey: [SECRETS_QUERY_KEY, projectId],
    enabled: !!projectId,
    queryFn: async () => (await listSecrets.execute(projectId)).unwrap(),
    staleTime: 30 * 1000,
  });

  return {
    secrets: data ?? [],
    isLoading,
    isError,
    error,
  };
};

/** Create a project-scoped secret. The value is write-only — sent once, never kept. */
export const useCreateSecret = (projectId: string) => {
  const { createSecret } = useDependencies().secret;
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (input: { name: string; value: string; description: string }) =>
      (
        await createSecret.execute({
          projectId,
          name: input.name,
          value: input.value,
          description: input.description,
        })
      ).unwrap(),
    onSuccess: () => {
      toast.success('Secret created');
      queryClient.invalidateQueries({ queryKey: [SECRETS_QUERY_KEY, projectId] });
    },
    onError: error => toast.error(secretErrorMessage(error)),
  });
};

/** Delete a project-scoped secret. */
export const useDeleteSecret = (projectId: string) => {
  const { deleteSecret } = useDependencies().secret;
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (secretId: string) => (await deleteSecret.execute(secretId)).unwrap(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [SECRETS_QUERY_KEY, projectId] });
    },
    onError: error => toast.error(secretErrorMessage(error)),
  });
};
