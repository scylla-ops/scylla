import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useRolesDomain } from '@/modules/features/roles/presentation/hooks/use-roles-domain.ts';
import type { PermissionScope } from '@platform/authz/domain/structs/permission.struct.ts';
import type {
  CreateGrantInput,
  RevokeAllAccessInput,
} from '@/modules/features/roles/domain/repository/permission.repository.ts';
import { useRefreshMyPermissions } from '@/modules/features/roles/presentation/hooks/use-refresh-my-permissions.ts';

/** Prefix shared by every grant list, so one mutation invalidates them all. */
const GRANTS_QUERY_ROOT = 'permission-grants';

/**
 * `undefined` scope is the system-wide list, which only a system administrator
 * may read; a scope narrows it to one organization or project, readable by an
 * administrator of that scope. The two are different requests with different
 * permissions, so they are different keys.
 */
export const GRANTS_QUERY_KEY = (scope?: PermissionScope, scopeId?: string) =>
  [GRANTS_QUERY_ROOT, scope ?? 'all', scopeId ?? ''] as const;

/**
 * Create / revoke, shared by every grant list. Each one invalidates the whole
 * grant prefix: a grant created from the project view also changes the
 * organization's list, and neither view knows the other exists.
 */
const useGrantMutations = () => {
  const { permissionRepository } = useRolesDomain();
  const queryClient = useQueryClient();
  const refreshMyPermissions = useRefreshMyPermissions();

  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: [GRANTS_QUERY_ROOT] });
    // A grant change may alter what the current user can see — reload the
    // stored effective permissions that drive nav/button gating.
    void refreshMyPermissions();
  };

  const createGrant = useMutation({
    mutationFn: async (input: CreateGrantInput) =>
      (await permissionRepository.createGrant(input)).unwrap(),
    onSuccess: invalidate,
  });

  const revokeGrant = useMutation({
    mutationFn: async (id: string) => (await permissionRepository.revokeGrant(id)).unwrap(),
    onSuccess: invalidate,
  });

  /**
   * Clears a principal's grants at a scope and beneath it. Distinct from
   * `revokeGrant`, which drops one grant by id: this is the "remove them from
   * here entirely" operation, and the only one that leaves no inert
   * project-scoped grant behind.
   */
  const revokeAllAccess = useMutation({
    mutationFn: async (input: RevokeAllAccessInput) =>
      (await permissionRepository.revokeAllAccess(input)).unwrap(),
    onSuccess: invalidate,
  });

  return { createGrant, revokeGrant, revokeAllAccess };
};

/**
 * Every grant in the installation (system-admin view) plus create/revoke
 * mutations. Anyone administering a single organization or project wants
 * {@link useScopedGrants} instead — this request is refused to them.
 */
export function useGrants() {
  const { permissionRepository } = useRolesDomain();
  const mutations = useGrantMutations();

  const grantsQuery = useQuery({
    queryKey: GRANTS_QUERY_KEY(),
    queryFn: async () => (await permissionRepository.listGrants()).unwrap(),
  });

  return {
    grants: grantsQuery.data ?? [],
    isLoading: grantsQuery.isLoading,
    isError: grantsQuery.isError,
    error: grantsQuery.error,
    ...mutations,
  };
}

/**
 * The grants bound to one organization or project — what a tenant administrator
 * may read, and the backing of the member views.
 *
 * `scopeId` may be `null` while the caller is still resolving which scope it is
 * looking at; the query simply stays idle. `enabled` is the caller's own gate:
 * the backend answers this only to a holder of `MANAGE_*_GRANTS` on the scope,
 * so a view that already knows the user lacks it should not ask and collect a
 * denial toast for an answer it never needed.
 */
export function useScopedGrants(
  scope: PermissionScope,
  scopeId: string | null,
  options: { enabled?: boolean } = {},
) {
  const { permissionRepository } = useRolesDomain();
  const mutations = useGrantMutations();
  const { enabled = true } = options;

  const grantsQuery = useQuery({
    queryKey: GRANTS_QUERY_KEY(scope, scopeId ?? ''),
    queryFn: async () => (await permissionRepository.listGrants(scope, scopeId!)).unwrap(),
    enabled: enabled && !!scopeId,
  });

  return {
    grants: grantsQuery.data ?? [],
    isLoading: grantsQuery.isLoading,
    isError: grantsQuery.isError,
    error: grantsQuery.error,
    ...mutations,
  };
}
