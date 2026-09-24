import { getQueryClient, mutationOptions, queryOptions } from '@platform/query';
import { getModuleDomain } from '@platform/di';
import { contextStore } from '@platform/context';
import { i18n } from '@lingui/core';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import type { OrganizationModule } from '../organization.module.ts';

// Resolved per call: tests swap the registry.
const repository = () =>
  getModuleDomain<typeof OrganizationModule.domain>('organization').organizationRepository;

export const ORGANIZATIONS_QUERY_KEY = () => ['organizations'] as const;
export const MY_ORGANIZATIONS_QUERY_KEY = () => ['organizations', 'mine'] as const;
export const ORGANIZATION_MEMBERS_QUERY_KEY = (organizationId: string) =>
  ['organizations', organizationId, 'members'] as const;

export const organizationQueries = {
  /** Not the global list: non-admins may not read it. */
  mine: () =>
    queryOptions({
      queryKey: MY_ORGANIZATIONS_QUERY_KEY(),
      queryFn: async () => (await repository().getMine()).unwrap(),
      staleTime: 1000 * 60 * 5, // 5 minutes TODO: change
    }),

  /** Derived from grants: callers invalidate it with `invalidateOrganizationMembers`. */
  members: (organizationId: string | null, options: { enabled?: boolean } = {}) =>
    queryOptions({
      queryKey: ORGANIZATION_MEMBERS_QUERY_KEY(organizationId ?? ''),
      queryFn: async () => (await repository().listMembers(organizationId!)).unwrap(),
      enabled: (options.enabled ?? true) && !!organizationId,
    }),
};

export const invalidateOrganizationMembers = (organizationId: string | null): void => {
  if (!organizationId) return;
  void getQueryClient().invalidateQueries({
    queryKey: ORGANIZATION_MEMBERS_QUERY_KEY(organizationId),
    exact: true,
  });
};

const invalidateOrganizations = () =>
  getQueryClient().invalidateQueries({ queryKey: ORGANIZATIONS_QUERY_KEY() });

export const organizationMutations = {
  create: () =>
    mutationOptions({
      mutationFn: async ({ name, description }: { name: string; description?: string }) =>
        (await repository().create(name, description)).unwrap(),
      onSuccess: data => {
        // The new organization becomes the active one.
        contextStore.getState().setOrganization(data.id, data.name);
        toast.success(i18n._(ToastMessages.ORGANIZATION_CREATE));
        return invalidateOrganizations();
      },
    }),

  update: () =>
    mutationOptions({
      mutationFn: async ({
        organizationId,
        name,
        description,
      }: {
        organizationId: string;
        name?: string;
        description?: string;
      }) => (await repository().update(organizationId, name, description)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.ORGANIZATION_UPDATE));
        return invalidateOrganizations();
      },
    }),

  remove: () =>
    mutationOptions({
      mutationFn: async (organizationId: string) =>
        (await repository().delete(organizationId)).unwrap(),
      onSuccess: () => {
        toast.success(i18n._(ToastMessages.ORGANIZATION_DELETE));
        return invalidateOrganizations();
      },
    }),
};
