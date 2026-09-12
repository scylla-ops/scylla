import { describe, it, expect } from 'vitest';
import { PermissionScope, PrincipalKind } from '@platform/authz';
import { buildOrganizationMembers, buildProjectMembers, MemberRoleOrigin } from './scope-member.struct';
import type { GrantEntity } from '@/modules/features/roles';

const userGrant = (overrides: Partial<GrantEntity> = {}): GrantEntity => ({
  id: 'grant-1',
  principal: { kind: PrincipalKind.USER, id: 'user-1' },
  roleId: 'role-1',
  scope: PermissionScope.ORGANIZATION,
  scopeId: 'org-1',
  ...overrides,
});

describe('buildOrganizationMembers', () => {
  it('lists a user holding an organization grant, with that grant as a DIRECT role', () => {
    const members = buildOrganizationMembers([userGrant()]);
    expect(members).toEqual([
      {
        userId: 'user-1',
        roles: [{ grantId: 'grant-1', roleId: 'role-1', origin: MemberRoleOrigin.DIRECT, scope: PermissionScope.ORGANIZATION }],
      },
    ]);
  });

  it('excludes app-principal grants - only USER principals are members', () => {
    const members = buildOrganizationMembers([
      userGrant({ principal: { kind: PrincipalKind.APP, id: 'app-1' } }),
    ]);
    expect(members).toEqual([]);
  });

  it('a known user id with no grant at all is still listed, with an empty role list', () => {
    const members = buildOrganizationMembers([], ['user-2']);
    expect(members).toEqual([{ userId: 'user-2', roles: [] }]);
  });

  it('merges a known user id with their own grants into one row, not two', () => {
    const members = buildOrganizationMembers([userGrant({ principal: { kind: PrincipalKind.USER, id: 'user-1' } })], [
      'user-1',
    ]);
    expect(members).toHaveLength(1);
    expect(members[0].roles).toHaveLength(1);
  });

  it('one user holding two grants gets both roles listed under the same row', () => {
    const members = buildOrganizationMembers([
      userGrant({ id: 'grant-1', roleId: 'role-a' }),
      userGrant({ id: 'grant-2', roleId: 'role-b' }),
    ]);
    expect(members).toHaveLength(1);
    expect(members[0].roles.map(r => r.roleId).sort()).toEqual(['role-a', 'role-b']);
  });
});

describe('buildProjectMembers', () => {
  const projectGrant = (overrides: Partial<GrantEntity> = {}): GrantEntity => ({
    id: 'p-grant-1',
    principal: { kind: PrincipalKind.USER, id: 'user-1' },
    roleId: 'project-role',
    scope: PermissionScope.PROJECT,
    scopeId: 'project-1',
    ...overrides,
  });

  it('lists a direct project grant as DIRECT/PROJECT', () => {
    const members = buildProjectMembers([projectGrant()], [], () => true);
    expect(members).toEqual([
      {
        userId: 'user-1',
        roles: [{ grantId: 'p-grant-1', roleId: 'project-role', origin: MemberRoleOrigin.DIRECT, scope: PermissionScope.PROJECT }],
      },
    ]);
  });

  it('includes an organization grant as INHERITED/ORGANIZATION when the predicate says it reaches projects', () => {
    const members = buildProjectMembers([], [userGrant({ roleId: 'org-admin' })], () => true);
    expect(members).toEqual([
      {
        userId: 'user-1',
        roles: [{ grantId: 'grant-1', roleId: 'org-admin', origin: MemberRoleOrigin.INHERITED, scope: PermissionScope.ORGANIZATION }],
      },
    ]);
  });

  it('excludes an organization grant whose role does not reach into projects (e.g. the bare "member" floor)', () => {
    const members = buildProjectMembers([], [userGrant({ roleId: 'organization-member' })], () => false);
    expect(members).toEqual([]);
  });

  it('combines a direct project role and an inherited organization role for the same user into one row', () => {
    const members = buildProjectMembers(
      [projectGrant()],
      [userGrant({ id: 'org-grant', roleId: 'org-admin' })],
      () => true,
    );
    expect(members).toHaveLength(1);
    expect(members[0].roles).toHaveLength(2);
    expect(members[0].roles.map(r => r.origin).sort()).toEqual(['direct', 'inherited']);
  });

  it('seeds known user ids the same way the organization builder does', () => {
    const members = buildProjectMembers([], [], () => true, ['user-9']);
    expect(members).toEqual([{ userId: 'user-9', roles: [] }]);
  });

  it('excludes app-principal grants from both project and organization sources', () => {
    const members = buildProjectMembers(
      [projectGrant({ principal: { kind: PrincipalKind.APP, id: 'app-1' } })],
      [userGrant({ principal: { kind: PrincipalKind.APP, id: 'app-1' } })],
      () => true,
    );
    expect(members).toEqual([]);
  });
});
