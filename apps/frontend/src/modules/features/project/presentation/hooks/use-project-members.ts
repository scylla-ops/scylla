import { useCallback } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useProjectDomain } from '@/modules/features/project/presentation/hooks/use-project-domain.ts';

export const PROJECT_MEMBERS_QUERY_KEY = (projectId: string) =>
  ['projects', projectId, 'members'] as const;

/**
 * Who holds a role scoped to the project.
 *
 * The list is derived from grants on the backend, so any grant mutation on this
 * project changes it. Mutations live in `useScopedGrants`, which cannot reach
 * this key without coupling the two features — callers that create or revoke a
 * grant therefore call `refetchMembers` themselves.
 */
export const useProjectMembers = (
  projectId: string | null,
  options: { enabled?: boolean } = {},
) => {
  const { projectRepository } = useProjectDomain();
  const queryClient = useQueryClient();
  const { enabled = true } = options;

  const {
    data: members,
    isLoading,
    error,
  } = useQuery({
    queryKey: PROJECT_MEMBERS_QUERY_KEY(projectId ?? ''),
    queryFn: async () => (await projectRepository.listMembers(projectId!)).unwrap(),
    enabled: enabled && !!projectId,
  });

  const refetchMembers = useCallback(() => {
    if (!projectId) return;
    void queryClient.invalidateQueries({
      queryKey: PROJECT_MEMBERS_QUERY_KEY(projectId),
      exact: true,
    });
  }, [queryClient, projectId]);

  return {
    members: members ?? [],
    isLoading,
    isError: !!error,
    error,
    refetchMembers,
  };
};
