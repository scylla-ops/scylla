import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { i18n } from '@lingui/core';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { SecretEntity } from '../domain/entities/secret.entity.ts';
import type { SecretModule } from '../secret.module.ts';

/**
 * Every read and write this module performs, declared as plain data.
 *
 * This is what `use-secrets.ts` was — three hooks — with the framework taken
 * out: `queryOptions` and `mutationOptions` describe the call, `createQuery` /
 * `createMutation` run it inside a component, and a test can exercise the
 * `queryFn` on its own.
 *
 * The repository is resolved per call, never at module load: the registry is
 * installed by the composition root and swapped by tests.
 */
const repository = () =>
  getModuleDomain<typeof SecretModule.domain>('secret').secretRepository;

export const SECRETS_QUERY_KEY = (projectId: string) => ['secrets', projectId] as const;

export interface CreateSecretValues {
  name: string;
  value: string;
  description: string;
}

export const secretQueries = {
  /** A project's secrets — **metadata only**, the backend never returns a value. */
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
  /** The plaintext passes through here once, on its way out. Never stored. */
  create: (projectId: string) =>
    mutationOptions({
      mutationFn: async ({ name, value, description }: CreateSecretValues) =>
        (await repository().create({ projectId, name, value, description })).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.SECRET_CREATE));
        void invalidateProject(projectId);
      },
    }),

  /**
   * Deleting breaks any running pipeline that depends on the secret — every
   * caller confirms first through `ConfirmOperationAlertDialog`.
   *
   * No success toast here: the two call sites differ. The row action reports
   * one deletion, the header's bulk delete reports the count.
   */
  remove: (projectId: string) =>
    mutationOptions({
      mutationFn: async (secretId: string) =>
        (await repository().deleteById(secretId)).unwrap(),
      onSuccess: () => void invalidateProject(projectId),
    }),
};
