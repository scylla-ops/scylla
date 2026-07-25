import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';

export const ORGANIZATION_MEMBERS_QUERY_KEY = (organizationId: string) =>
  ['organization-members', organizationId] as const;

/**
 * Members of a single organization plus add/remove mutations. Server state only
 * — the member list lives in TanStack Query, keyed by the organization id.
 */
export const useOrganizationMembers = (organizationId: string | null) => {
  const { listOrganizationMembers, addOrganizationMember, removeOrganizationMember } =
    useDependencies().organization;
  const queryClient = useQueryClient();
  const { i18n } = useLingui();

  const invalidate = () =>
    queryClient.invalidateQueries({
      queryKey: ORGANIZATION_MEMBERS_QUERY_KEY(organizationId ?? ''),
      exact: true,
    });

  const membersQuery = useQuery({
    queryKey: ORGANIZATION_MEMBERS_QUERY_KEY(organizationId ?? ''),
    queryFn: async () => (await listOrganizationMembers.execute(organizationId!)).unwrap(),
    enabled: !!organizationId,
  });

  const addMember = useMutation({
    mutationFn: async (userId: string) =>
      (await addOrganizationMember.execute(organizationId!, userId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.ORGANIZATION_MEMBER_ADD));
      return invalidate();
    },
  });

  const removeMember = useMutation({
    mutationFn: async (userId: string) =>
      (await removeOrganizationMember.execute(organizationId!, userId)).unwrap(),
    onSuccess: () => {
      toast.success(i18n._(ToastMessages.ORGANIZATION_MEMBER_REMOVE));
      return invalidate();
    },
  });

  return {
    members: membersQuery.data ?? [],
    isLoading: membersQuery.isLoading,
    isError: membersQuery.isError,
    addMember,
    removeMember,
  };
};
