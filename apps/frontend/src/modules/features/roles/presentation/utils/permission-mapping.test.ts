import { describe, it, expect } from 'vitest';
import { Permission, PermissionScope } from '@platform/authz';
import {
  PERMISSION_CATALOG,
  getPermissionDefinition,
  isEditablePermission,
  getPermissionDefinitionsForScope,
  getAlwaysGrantedPermissionsForScope,
  isHiddenAtScope,
  withImplicitPermissions,
  getEditablePermissionDefinitionsForScope,
  getPermissionsForScope,
  humanizePermission,
} from './permission-mapping';

describe('catalog lookups', () => {
  it('getPermissionDefinition finds a cataloged permission', () => {
    expect(getPermissionDefinition(Permission.CREATE_PROJECT)?.scope).toBe(PermissionScope.ORGANIZATION);
  });

  it('getPermissionDefinition returns undefined for a permission outside the V1 catalog', () => {
    // READ_PIPELINE's own dependency graph is catalogued, but a raw wire
    // permission the UI never gates on (e.g. an invitation permission) is not.
    expect(getPermissionDefinition(Permission.UNSPECIFIED)).toBeUndefined();
  });

  it('isEditablePermission mirrors the catalog membership', () => {
    expect(isEditablePermission(Permission.LIST_SECRETS)).toBe(true);
    expect(isEditablePermission(Permission.UNSPECIFIED)).toBe(false);
  });

  it('every catalog entry\'s dependsOn (if any) is itself in the catalog — no dangling tree parent', () => {
    const ids = new Set(PERMISSION_CATALOG.map(d => d.id));
    for (const definition of PERMISSION_CATALOG) {
      if (definition.dependsOn !== undefined) {
        expect(ids.has(definition.dependsOn)).toBe(true);
      }
    }
  });
});

describe('getPermissionDefinitionsForScope — the scope hierarchy', () => {
  it('a PROJECT-scoped role only sees project-scoped permissions', () => {
    const scopes = getPermissionDefinitionsForScope(PermissionScope.PROJECT).map(d => d.scope);
    expect(scopes.every(s => s === PermissionScope.PROJECT)).toBe(true);
  });

  it('an ORGANIZATION-scoped role sees organization AND project permissions, never system', () => {
    const scopes = getPermissionDefinitionsForScope(PermissionScope.ORGANIZATION).map(d => d.scope);
    expect(scopes).toContain(PermissionScope.ORGANIZATION);
    expect(scopes).toContain(PermissionScope.PROJECT);
    expect(scopes).not.toContain(PermissionScope.SYSTEM);
  });

  it('a SYSTEM-scoped role sees every scope', () => {
    const scopes = new Set(getPermissionDefinitionsForScope(PermissionScope.SYSTEM).map(d => d.scope));
    expect(scopes).toEqual(new Set([PermissionScope.SYSTEM, PermissionScope.ORGANIZATION, PermissionScope.PROJECT]));
  });

  it('UNSPECIFIED scope confers nothing', () => {
    expect(getPermissionDefinitionsForScope(PermissionScope.UNSPECIFIED)).toEqual([]);
  });

  it('getPermissionsForScope is the same set, reduced to ids', () => {
    const definitions = getPermissionDefinitionsForScope(PermissionScope.PROJECT);
    expect(getPermissionsForScope(PermissionScope.PROJECT)).toEqual(definitions.map(d => d.id));
  });
});

describe('implicit permissions — the "member of" / "manage grants" rule', () => {
  it('SYSTEM grants nothing unconditionally (MANAGE_SYSTEM_GRANTS rides on MANAGE_ROLES, it is not free)', () => {
    expect(getAlwaysGrantedPermissionsForScope(PermissionScope.SYSTEM)).toEqual([]);
  });

  it('ORGANIZATION unconditionally grants READ_ORGANIZATION only (membership itself)', () => {
    expect(getAlwaysGrantedPermissionsForScope(PermissionScope.ORGANIZATION)).toEqual([
      Permission.READ_ORGANIZATION,
    ]);
  });

  it('PROJECT grants nothing unconditionally (READ_PIPELINE rides on LIST_PIPELINES_BY_PROJECT there)', () => {
    expect(getAlwaysGrantedPermissionsForScope(PermissionScope.PROJECT)).toEqual([]);
  });

  it('isHiddenAtScope: MANAGE_SYSTEM_GRANTS is hidden at SYSTEM but a real, visible toggle elsewhere', () => {
    expect(isHiddenAtScope(Permission.MANAGE_SYSTEM_GRANTS, PermissionScope.SYSTEM)).toBe(true);
    expect(isHiddenAtScope(Permission.MANAGE_SYSTEM_GRANTS, PermissionScope.ORGANIZATION)).toBe(false);
  });

  it('isHiddenAtScope: READ_PROJECT is hidden at ORGANIZATION (stands in for LIST_PROJECTS_BY_ORGANIZATION) but not at PROJECT, where it is the whole point', () => {
    expect(isHiddenAtScope(Permission.READ_PROJECT, PermissionScope.ORGANIZATION)).toBe(true);
    expect(isHiddenAtScope(Permission.READ_PROJECT, PermissionScope.PROJECT)).toBe(false);
  });
});

describe('withImplicitPermissions — what actually gets written', () => {
  it('SYSTEM: ticking MANAGE_ROLES writes MANAGE_SYSTEM_GRANTS too', () => {
    const written = withImplicitPermissions(PermissionScope.SYSTEM, [Permission.MANAGE_ROLES]);
    expect(written).toContain(Permission.MANAGE_ROLES);
    expect(written).toContain(Permission.MANAGE_SYSTEM_GRANTS);
  });

  it('SYSTEM: without MANAGE_ROLES, MANAGE_SYSTEM_GRANTS is never added for free', () => {
    const written = withImplicitPermissions(PermissionScope.SYSTEM, [Permission.LIST_USERS]);
    expect(written).not.toContain(Permission.MANAGE_SYSTEM_GRANTS);
  });

  it('ORGANIZATION: READ_ORGANIZATION is always written, even from an empty selection', () => {
    expect(withImplicitPermissions(PermissionScope.ORGANIZATION, [])).toEqual([
      Permission.READ_ORGANIZATION,
    ]);
  });

  it('ORGANIZATION: ticking LIST_PROJECTS_BY_ORGANIZATION also writes READ_PROJECT, but not READ_PIPELINE', () => {
    const written = withImplicitPermissions(PermissionScope.ORGANIZATION, [
      Permission.LIST_PROJECTS_BY_ORGANIZATION,
    ]);
    expect(written).toEqual(
      expect.arrayContaining([
        Permission.READ_ORGANIZATION,
        Permission.LIST_PROJECTS_BY_ORGANIZATION,
        Permission.READ_PROJECT,
      ]),
    );
    expect(written).not.toContain(Permission.READ_PIPELINE);
  });

  it('ORGANIZATION: ticking LIST_PIPELINES_BY_PROJECT writes READ_PIPELINE', () => {
    const written = withImplicitPermissions(PermissionScope.ORGANIZATION, [
      Permission.LIST_PIPELINES_BY_PROJECT,
    ]);
    expect(written).toContain(Permission.READ_PIPELINE);
  });

  it('PROJECT: ticking LIST_PIPELINES_BY_PROJECT writes READ_PIPELINE there too', () => {
    const written = withImplicitPermissions(PermissionScope.PROJECT, [
      Permission.LIST_PIPELINES_BY_PROJECT,
    ]);
    expect(written).toContain(Permission.READ_PIPELINE);
  });

  it('never duplicates a permission that is both explicitly selected and implicitly conferred', () => {
    const written = withImplicitPermissions(PermissionScope.ORGANIZATION, [
      Permission.READ_ORGANIZATION,
    ]);
    expect(written.filter(p => p === Permission.READ_ORGANIZATION)).toHaveLength(1);
  });
});

describe('getEditablePermissionDefinitionsForScope — what the editor actually renders', () => {
  it('never renders a hidden (implicit) permission as a toggle', () => {
    const ids = getEditablePermissionDefinitionsForScope(PermissionScope.ORGANIZATION).map(d => d.id);
    expect(ids).not.toContain(Permission.READ_ORGANIZATION);
    expect(ids).not.toContain(Permission.READ_PROJECT);
    expect(ids).not.toContain(Permission.READ_PIPELINE);
  });

  it('re-parents a child of a hidden node onto its stand-in, so the tree does not point at a node that is not rendered', () => {
    const definitions = getEditablePermissionDefinitionsForScope(PermissionScope.ORGANIZATION);
    const updateProject = definitions.find(d => d.id === Permission.UPDATE_PROJECT);
    // UPDATE_PROJECT's cataloged parent is READ_PROJECT, which is hidden at
    // ORGANIZATION scope and stands in for LIST_PROJECTS_BY_ORGANIZATION.
    expect(updateProject?.dependsOn).toBe(Permission.LIST_PROJECTS_BY_ORGANIZATION);
  });

  it('leaves dependsOn untouched when the parent is not hidden at this scope', () => {
    const definitions = getEditablePermissionDefinitionsForScope(PermissionScope.PROJECT);
    const updateProject = definitions.find(d => d.id === Permission.UPDATE_PROJECT);
    // At PROJECT scope, READ_PROJECT itself is the visible root — not hidden.
    expect(updateProject?.dependsOn).toBe(Permission.READ_PROJECT);
  });

  it('is exactly getPermissionDefinitionsForScope minus the hidden entries, same order', () => {
    const all = getPermissionDefinitionsForScope(PermissionScope.PROJECT).map(d => d.id);
    const editable = getEditablePermissionDefinitionsForScope(PermissionScope.PROJECT).map(d => d.id);
    const hiddenIds = all.filter(id => isHiddenAtScope(id, PermissionScope.PROJECT));
    expect(editable).toEqual(all.filter(id => !hiddenIds.includes(id)));
  });
});

describe('humanizePermission', () => {
  it('turns an enum key into a readable, capitalized phrase', () => {
    expect(humanizePermission(Permission.LIST_JOBS_BY_PIPELINE)).toBe('List jobs by pipeline');
  });

  it('falls back to a numeric label for a value with no enum key (e.g. a future/unknown wire value)', () => {
    expect(humanizePermission(9999 as Permission)).toBe('Permission #9999');
  });
});
