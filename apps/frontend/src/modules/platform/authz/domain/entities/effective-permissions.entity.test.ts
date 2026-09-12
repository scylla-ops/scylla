import { describe, it, expect } from 'vitest';
import { canAccess } from './effective-permissions.entity';
import type { EffectivePermissionsEntity } from './effective-permissions.entity';
import { Permission, PermissionScope } from '@platform/authz/domain/structs/permission.struct.ts';

const effective = (...scopes: EffectivePermissionsEntity['scopes']): EffectivePermissionsEntity => ({
  scopes,
});

describe('canAccess', () => {
  it('denies when there are no effective scopes at all', () => {
    expect(canAccess(effective(), Permission.READ_PROJECT)).toBe(false);
  });

  it('a SYSTEM fullControl scope grants anything, anywhere, even with no target given', () => {
    const perms = effective({ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } });
    expect(canAccess(perms, Permission.READ_PROJECT)).toBe(true);
    expect(canAccess(perms, Permission.DELETE_ORGANIZATION, { organizationId: 'org-1' })).toBe(true);
  });

  it('a restricted access confers only the permissions it explicitly lists', () => {
    const perms = effective({
      scope: PermissionScope.SYSTEM,
      scopeId: '',
      access: { kind: 'restricted', permissions: [Permission.READ_PROJECT] },
    });
    expect(canAccess(perms, Permission.READ_PROJECT)).toBe(true);
    expect(canAccess(perms, Permission.UPDATE_PROJECT)).toBe(false);
  });

  it('an "unknown" access arm never confers anything, even at SYSTEM scope', () => {
    const perms = effective({ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'unknown' } });
    expect(canAccess(perms, Permission.READ_PROJECT)).toBe(false);
  });

  describe('IMPLIED_BY (MANAGE_ROLES implies MANAGE_SYSTEM_GRANTS)', () => {
    it('holding MANAGE_ROLES also confers MANAGE_SYSTEM_GRANTS', () => {
      const perms = effective({
        scope: PermissionScope.SYSTEM,
        scopeId: '',
        access: { kind: 'restricted', permissions: [Permission.MANAGE_ROLES] },
      });
      expect(canAccess(perms, Permission.MANAGE_SYSTEM_GRANTS)).toBe(true);
    });

    it('the implication is one-directional: holding MANAGE_SYSTEM_GRANTS does not confer MANAGE_ROLES', () => {
      const perms = effective({
        scope: PermissionScope.SYSTEM,
        scopeId: '',
        access: { kind: 'restricted', permissions: [Permission.MANAGE_SYSTEM_GRANTS] },
      });
      expect(canAccess(perms, Permission.MANAGE_ROLES)).toBe(false);
    });
  });

  describe('ORGANIZATION scope', () => {
    const orgGrant = (orgId: string) =>
      effective({
        scope: PermissionScope.ORGANIZATION,
        scopeId: orgId,
        access: { kind: 'fullControl' },
      });

    it('covers a target in that same organization', () => {
      expect(canAccess(orgGrant('org-1'), Permission.READ_PROJECT, { organizationId: 'org-1' })).toBe(true);
    });

    it('does not cover a different organization', () => {
      expect(canAccess(orgGrant('org-1'), Permission.READ_PROJECT, { organizationId: 'org-2' })).toBe(false);
    });

    it('does not cover a target with no organizationId given', () => {
      expect(canAccess(orgGrant('org-1'), Permission.READ_PROJECT)).toBe(false);
    });

    it('does not cover a target scoped only by projectId', () => {
      expect(canAccess(orgGrant('org-1'), Permission.READ_PROJECT, { projectId: 'project-1' })).toBe(false);
    });
  });

  describe('PROJECT scope', () => {
    const projectGrant = (projectId: string) =>
      effective({
        scope: PermissionScope.PROJECT,
        scopeId: projectId,
        access: { kind: 'fullControl' },
      });

    it('covers that exact project', () => {
      expect(canAccess(projectGrant('project-1'), Permission.READ_PROJECT, { projectId: 'project-1' })).toBe(
        true,
      );
    });

    it('does not cover a different project', () => {
      expect(canAccess(projectGrant('project-1'), Permission.READ_PROJECT, { projectId: 'project-2' })).toBe(
        false,
      );
    });

    it('does not extend to the project\'s own organization - a project grant is narrower, not broader', () => {
      expect(
        canAccess(projectGrant('project-1'), Permission.READ_PROJECT, { organizationId: 'org-1' }),
      ).toBe(false);
    });
  });

  it('an UNSPECIFIED (or otherwise unrecognized) scope entry never covers any target', () => {
    const perms = effective({
      scope: PermissionScope.UNSPECIFIED,
      scopeId: '',
      access: { kind: 'fullControl' },
    });
    expect(canAccess(perms, Permission.READ_PROJECT, { organizationId: 'org-1', projectId: 'project-1' })).toBe(
      false,
    );
  });

  it('checks every scope and grants access as soon as any one of them matches', () => {
    const perms = effective(
      { scope: PermissionScope.PROJECT, scopeId: 'project-2', access: { kind: 'fullControl' } },
      { scope: PermissionScope.PROJECT, scopeId: 'project-1', access: { kind: 'fullControl' } },
    );
    expect(canAccess(perms, Permission.READ_PROJECT, { projectId: 'project-1' })).toBe(true);
  });
});
