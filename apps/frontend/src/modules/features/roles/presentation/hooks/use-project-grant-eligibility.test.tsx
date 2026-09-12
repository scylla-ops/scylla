import { describe, it, expect, vi } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { DependenciesProvider } from '@platform/di';
import { PermissionScope, PrincipalKind, Permission } from '@platform/authz';
import { ScyllaResult } from '@shared/utils/scylla-result.ts';
import { useProjectGrantEligibility } from './use-project-grant-eligibility';
import { UpdateRoleUseCase } from '@/modules/features/roles/domain/use-cases/update-role.use-case.ts';
import type { PermissionRepository } from '@/modules/features/roles/domain/repository/permission.repository.ts';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/roles/domain/entities/grant.entity.ts';

const ORG_ID = 'org-1';

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
  id: 'role-1',
  name: 'Org member',
  description: '',
  scope: PermissionScope.ORGANIZATION,
  origin: { kind: 'builtin', key: 'organization-member' },
  access: { kind: 'restricted', permissions: [Permission.READ_ORGANIZATION] },
  ...overrides,
});

const grant = (overrides: Partial<GrantEntity> = {}): GrantEntity => ({
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'role-1',
  scope: PermissionScope.ORGANIZATION,
  scopeId: ORG_ID,
  ...overrides,
});

const makeFakeRepository = (grants: GrantEntity[], roles: RoleEntity[]): PermissionRepository => {
  const noop = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  return {
    listRoles: vi.fn().mockResolvedValue(ScyllaResult.success(roles)),
    getRole: noop,
    createRole: noop,
    updateRole: noop,
    deleteRole: noop,
    getEffectivePermissions: noop,
    getMyPermissions: noop,
    listGrants: vi.fn().mockResolvedValue(ScyllaResult.success(grants)),
    createGrant: noop,
    revokeGrant: noop,
    revokeAllAccess: noop,
    listGrantableRoles: vi.fn().mockResolvedValue(ScyllaResult.success([])),
    listPermissionVocabulary: vi.fn().mockResolvedValue(ScyllaResult.success({ actions: [] })),
  };
};

const wrapperFor = (repository: PermissionRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <DependenciesProvider
        registry={{ roles: { permissionRepository: repository, updateRole: new UpdateRoleUseCase(repository) } }}
      >
        {children}
      </DependenciesProvider>
    </QueryClientProvider>
  );
  return Wrapper;
};

describe('useProjectGrantEligibility', () => {
  it('a user with no grant at all in the organization is "not-admitted"', async () => {
    const repository = makeFakeRepository([], []);
    const { result } = renderHook(() => useProjectGrantEligibility(ORG_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.eligibilityFor('user-1')).toBe('not-admitted'));
  });

  it('a user holding an organization-scoped role that confers READ_ORGANIZATION is "eligible"', async () => {
    const repository = makeFakeRepository(
      [grant({ roleId: 'admin' })],
      [role({ id: 'admin', access: { kind: 'fullControl' } })],
    );
    const { result } = renderHook(() => useProjectGrantEligibility(ORG_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.eligibilityFor('user-1')).toBe('eligible'));
  });

  it('admitted but without READ_ORGANIZATION on any held role is "cannot-see-projects"', async () => {
    const repository = makeFakeRepository(
      [grant({ roleId: 'billing-only' })],
      [
        role({
          id: 'billing-only',
          access: { kind: 'restricted', permissions: [Permission.UPDATE_ORGANIZATION] },
        }),
      ],
    );
    const { result } = renderHook(() => useProjectGrantEligibility(ORG_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() =>
      expect(result.current.eligibilityFor('user-1')).toBe('cannot-see-projects'),
    );
  });

  it('a grant bound to a DIFFERENT organization does not count', async () => {
    const repository = makeFakeRepository(
      [grant({ scopeId: 'org-2' })],
      [role({ access: { kind: 'fullControl' } })],
    );
    const { result } = renderHook(() => useProjectGrantEligibility(ORG_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.eligibilityFor('user-1')).toBe('not-admitted'));
  });

  it('a SYSTEM-scoped grant does not count as organization membership either', async () => {
    const repository = makeFakeRepository(
      [grant({ scope: PermissionScope.SYSTEM, scopeId: '' })],
      [role({ access: { kind: 'fullControl' } })],
    );
    const { result } = renderHook(() => useProjectGrantEligibility(ORG_ID), {
      wrapper: wrapperFor(repository),
    });

    await waitFor(() => expect(result.current.eligibilityFor('user-1')).toBe('not-admitted'));
  });

  it('returns "not-admitted" for every user while organizationId is null', () => {
    const repository = makeFakeRepository([grant()], [role({ access: { kind: 'fullControl' } })]);
    const { result } = renderHook(() => useProjectGrantEligibility(null), {
      wrapper: wrapperFor(repository),
    });

    expect(result.current.eligibilityFor('user-1')).toBe('not-admitted');
  });
});
