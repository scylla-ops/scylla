import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { i18n } from '@lingui/core';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { SecretEntity } from '../domain/entities/secret.entity.ts';
import type { SecretModule } from '../secret.module.ts';

// Resolved per call: tests swap the registry.
const repository = () =>
  getModuleDomain<typeof SecretModule.domain>('secret').secretRepository;

export const SECRETS_QUERY_KEY = (projectId: string) => ['secrets', projectId] as const;

export interface CreateSecretValues {
  name: string;
  value: string;
  description: string;
}

export const secretQueries = {
  /** Metadata only. */
  byProject: (projectId: string) =>
    queryOptions<SecretEntity[]>({
      queryKey: SECRETS_QUERY_KEY(projectId),
      enabled: !!projectId,
      queryFn: async () => (await repository().listByProjectId(projectId)).unwrap(),
      staleTime: 30 * 1000,
    }),
};

const invalidateProject = (projectId: string) =>
  getQueryClient().invalidateQueries({ queryKey: SECRETS_QUERY_KEY(projectId) });

export const secretMutations = {
  /** The value passes through once and is never stored. */
  create: (projectId: string) =>
    mutationOptions({
      mutationFn: async ({ name, value, description }: CreateSecretValues) =>
        (await repository().create({ projectId, name, value, description })).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.SECRET_CREATE));
        void invalidateProject(projectId);
      },
    }),

  /** Breaks the pipelines that use it: confirm first. No success toast: each caller words its own. */
  remove: (projectId: string) =>
    mutationOptions({
      mutationFn: async (secretId: string) =>
        (await repository().deleteById(secretId)).unwrap(),
      onSuccess: () => void invalidateProject(projectId),
    }),
};
