import { useQuery } from '@tanstack/react-query';
import { useDependencies } from '@core/presentation/hooks/use-dependencies.ts';
import {
  type Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

const VOCAB_QUERY_KEY = 'authz-vocabulary';

/**
 * The permission vocabulary as a coherence oracle: `coherentAtScope(p, scope)`
 * tells whether permission `p` does anything in a role bound to `scope`. The
 * backend publishes each permission's minimal (narrowest) coherent scope; a
 * permission is usable at that scope or any broader (ancestor) one.
 *
 * `PermissionScope` values are ordered broad→narrow
 * (SYSTEM=1 < ORGANIZATION=2 < PROJECT=3), so "broader or equal" is
 * simply `scope <= minScope`.
 */
export function useAuthzVocabulary() {
  const { authz } = useDependencies();

  const query = useQuery({
    queryKey: [VOCAB_QUERY_KEY],
    queryFn: async () => (await authz.listAuthzVocabulary.execute()).unwrap(),
    staleTime: Infinity, // closed, code-owned catalog — never goes stale
  });

  const minScope = new Map<Permission, PermissionScope>();
  for (const action of query.data?.actions ?? []) {
    minScope.set(action.permission, action.minScope);
  }

  const coherentAtScope = (permission: Permission, scope: PermissionScope): boolean => {
    const min = minScope.get(permission);
    // Unknown (not loaded yet / absent) → don't block.
    if (min == null || min === PermissionScope.UNSPECIFIED) return true;
    return scope <= min;
  };

  return { coherentAtScope, isLoading: query.isLoading };
}
