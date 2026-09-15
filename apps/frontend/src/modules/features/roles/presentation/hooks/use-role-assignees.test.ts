import { describe, it, expect, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import { PrincipalKind } from '@platform/authz';
import { useRoleAssignees } from './use-role-assignees';
import type { RoleEntity } from '@/modules/features/roles/domain/entities/role.entity.ts';
import type { GrantEntity } from '@/modules/features/roles/domain/entities/grant.entity.ts';

const revokeGrantMutate = vi.fn();
const grantsFixture: GrantEntity[] = [];
vi.mock('@/modules/features/roles/presentation/hooks/use-grants.ts', () => ({
  useGrants: () => ({
    grants: grantsFixture,
    revokeGrant: { mutate: revokeGrantMutate, isPending: false },
  }),
}));

const usersFixture = { items: [{ userId: 'user-1', username: 'ravenne' }] };
vi.mock('@/modules/features/user', () => ({
  useUsers: () => ({ users: usersFixture }),
}));

const role = (overrides: Partial<RoleEntity> = {}): RoleEntity =>
  ({ id: 'role-1', name: 'CI runner', description: '', ...overrides }) as RoleEntity;

describe('useRoleAssignees', () => {
  it('returns no assignees when role is null', () => {
    const { result } = renderHook(() => useRoleAssignees(null));
    expect(result.current.assignees).toEqual([]);
  });

  it('filters grants down to the given role and resolves a user principal to its username', () => {
    grantsFixture.splice(
      0,
      grantsFixture.length,
      { id: 'g1', principal: { kind: PrincipalKind.USER, id: 'user-1' }, roleId: 'role-1', scope: 0, scopeId: '' },
      { id: 'g2', principal: { kind: PrincipalKind.USER, id: 'user-2' }, roleId: 'other-role', scope: 0, scopeId: '' },
    );

    const { result } = renderHook(() => useRoleAssignees(role()));

    expect(result.current.assignees).toHaveLength(1);
    expect(result.current.assignees[0]).toEqual({
      grant: grantsFixture[0],
      label: 'ravenne',
    });
  });

  it('falls back to the principal id when the user cannot be resolved (e.g. deleted since)', () => {
    grantsFixture.splice(0, grantsFixture.length, {
      id: 'g1',
      principal: { kind: PrincipalKind.USER, id: 'ghost-user' },
      roleId: 'role-1',
      scope: 0,
      scopeId: '',
    });

    const { result } = renderHook(() => useRoleAssignees(role()));
    expect(result.current.assignees[0].label).toBe('ghost-user');
  });

  it('an app principal is labeled by its own id, never looked up in the user directory', () => {
    grantsFixture.splice(0, grantsFixture.length, {
      id: 'g1',
      principal: { kind: PrincipalKind.APP, id: 'app-42' },
      roleId: 'role-1',
      scope: 0,
      scopeId: '',
    });

    const { result } = renderHook(() => useRoleAssignees(role()));
    expect(result.current.assignees[0].label).toBe('app-42');
  });

  it('removeAssignee revokes the grant by id', () => {
    const { result } = renderHook(() => useRoleAssignees(role()));
    result.current.removeAssignee('grant-to-remove');
    expect(revokeGrantMutate).toHaveBeenCalledWith('grant-to-remove');
  });
});
