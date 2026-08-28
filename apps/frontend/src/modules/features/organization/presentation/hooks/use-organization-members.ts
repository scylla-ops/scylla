import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback } from 'react';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';

export const ORGANIZATION_MEMBERS_QUERY_KEY = (organizationId: string) =>
  ['organizations', organizationId, 'members'] as const;

/**
 * Who belongs to an organization.
 *
 * The list is derived from grants on the backend, so any grant mutation on this
 * organization changes it. Mutations live in `useGrants`, which cannot reach
 * this key without coupling the two features — callers that create or revoke a
 * grant therefore call `refetchMembers` themselves.
 */
export const useOrganizationMembers = (
  organizationId: string | null,
  options: { enabled?: boolean } = {},
) => {
  const { listOrganizationMembers } = useDependencies().organization;
  const queryClient = useQueryClient();
  const { enabled = true } = options;

  const {
    data: members,
    isLoading,
    error,
  } = useQuery({
    queryKey: ORGANIZATION_MEMBERS_QUERY_KEY(organizationId ?? ''),
    queryFn: async () => (await listOrganizationMembers.execute(organizationId!)).unwrap(),
    enabled: enabled && !!organizationId,
  });

  const refetchMembers = useCallback(() => {
    if (!organizationId) return;
    void queryClient.invalidateQueries({
      queryKey: ORGANIZATION_MEMBERS_QUERY_KEY(organizationId),
      exact: true,
    });
  }, [queryClient, organizationId]);

  return {
    members: members ?? [],
    isLoading,
    isError: !!error,
    error,
    refetchMembers,
  };
};
