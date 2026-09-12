import { describe, it, expect } from 'vitest';
import { GrpcRoleMapper } from './grpc-role.mapper';
import { ScopeKind } from '@/generated/scylla/authz/v1/permission.ts';
import type { Role } from '@/generated/scylla/authz/v1/role.ts';
import { PermissionScope } from '@platform/authz';
import type { RoleEntity, RoleCreationData } from '@/modules/features/roles/domain/entities/role.entity.ts';

const baseRole = (overrides: Partial<Role> = {}): Role => ({
  roleId: { value: 'role-1' },
  name: 'CI runner',
  description: 'runs pipelines',
  scopeKind: ScopeKind.PROJECT,
  access: { access: { oneofKind: 'fullControl', fullControl: {} } },
  origin: { oneofKind: 'custom', custom: {} },
  ...overrides,
});

describe('GrpcRoleMapper.toDomain', () => {
  it('unwraps the id, carries name/description, and maps scope/access', () => {
    const domain = GrpcRoleMapper.toDomain(baseRole());
    expect(domain.id).toBe('role-1');
    expect(domain.name).toBe('CI runner');
    expect(domain.description).toBe('runs pipelines');
    expect(domain.scope).toBe(PermissionScope.PROJECT);
    expect(domain.access).toEqual({ kind: 'fullControl' });
  });

  it('defaults to an empty id when the wrapper is absent', () => {
    expect(GrpcRoleMapper.toDomain(baseRole({ roleId: undefined })).id).toBe('');
  });

  describe('origin', () => {
    it('maps a builtin role, carrying its stable key', () => {
      const domain = GrpcRoleMapper.toDomain(
        baseRole({ origin: { oneofKind: 'builtin', builtin: { key: 'organization-admin' } } }),
      );
      expect(domain.origin).toEqual({ kind: 'builtin', key: 'organization-admin' });
    });

    it('maps a custom role, carrying its owning organization', () => {
      const domain = GrpcRoleMapper.toDomain(
        baseRole({
          origin: { oneofKind: 'custom', custom: { ownerOrganizationId: { value: 'org-1' } } },
        }),
      );
      expect(domain.origin).toEqual({ kind: 'custom', ownerOrganizationId: 'org-1' });
    });

    it('a custom role with no owner id is still "custom", with an undefined owner', () => {
      const domain = GrpcRoleMapper.toDomain(baseRole({ origin: { oneofKind: 'custom', custom: {} } }));
      expect(domain.origin).toEqual({ kind: 'custom', ownerOrganizationId: undefined });
    });

    it('an origin arm this build does not know reads as unknown, never as "custom"', () => {
      const domain = GrpcRoleMapper.toDomain(baseRole({ origin: { oneofKind: undefined } }));
      expect(domain.origin).toEqual({ kind: 'unknown' });
    });
  });
});

describe('GrpcRoleMapper.toGrpcCreateRequest', () => {
  it('builds the wire create request from RoleCreationData', () => {
    const data: RoleCreationData = {
      name: 'CI runner',
      description: 'runs pipelines',
      scope: PermissionScope.PROJECT,
      access: { kind: 'fullControl' },
    };
    expect(GrpcRoleMapper.toGrpcCreateRequest(data)).toEqual({
      name: 'CI runner',
      description: 'runs pipelines',
      scopeKind: ScopeKind.PROJECT,
      access: { access: { oneofKind: 'fullControl', fullControl: {} } },
    });
  });
});

describe('GrpcRoleMapper.toGrpcUpdateRequest', () => {
  const role = (overrides: Partial<RoleEntity> = {}): RoleEntity => ({
    id: 'role-1',
    name: 'CI runner',
    description: 'runs pipelines',
    scope: PermissionScope.PROJECT,
    origin: { kind: 'custom' },
    access: { kind: 'fullControl' },
    ...overrides,
  });

  it('builds the wire update request (no scope - a role\'s scope is immutable)', () => {
    expect(GrpcRoleMapper.toGrpcUpdateRequest(role())).toEqual({
      roleId: { value: 'role-1' },
      name: 'CI runner',
      description: 'runs pipelines',
      access: { access: { oneofKind: 'fullControl', fullControl: {} } },
    });
  });

  it('refuses to update a role whose access this build cannot read - saving it back would drop it', () => {
    expect(() => GrpcRoleMapper.toGrpcUpdateRequest(role({ access: { kind: 'unknown' } }))).toThrow(
      /cannot be updated here/,
    );
  });
});
