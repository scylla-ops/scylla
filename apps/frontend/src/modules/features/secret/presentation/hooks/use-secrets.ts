import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useSecretDomain } from '@/modules/features/secret/presentation/hooks/use-secret-domain.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { ScyllaError } from '@shared/utils/scylla-result.ts';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';

const SECRETS_QUERY_KEY = 'secrets';

/** List a project's secrets (metadata only — never a value). */
export const useSecrets = (projectId: string) => {
  const { secretRepository } = useSecretDomain();

  const { data, isLoading, isError, error } = useQuery<SecretEntity[], ScyllaError>({
    queryKey: [SECRETS_QUERY_KEY, projectId],
    enabled: !!projectId,
    queryFn: async () => (await secretRepository.listByProjectId(projectId)).unwrap(),
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
  const { secretRepository } = useSecretDomain();
  const queryClient = useQueryClient();
  const { i18n } = useLingui();

  return useMutation({
    mutationFn: async (input: { name: string; value: string; description: string }) =>
      (
        await secretRepository.create({
          projectId,
          name: input.name,
          value: input.value,
          description: input.description,
        })
      ).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.SECRET_CREATE));
      void queryClient.invalidateQueries({ queryKey: [SECRETS_QUERY_KEY, projectId] });
    },
  });
};

/** Delete a project-scoped secret. */
export const useDeleteSecret = (projectId: string) => {
  const { secretRepository } = useSecretDomain();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (secretId: string) => (await secretRepository.deleteById(secretId)).unwrap(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: [SECRETS_QUERY_KEY, projectId] });
    },
  });
};
