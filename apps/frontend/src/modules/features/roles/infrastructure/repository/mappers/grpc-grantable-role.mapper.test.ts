import { describe, it, expect } from 'vitest';
import { GrpcGrantableRoleMapper } from './grpc-grantable-role.mapper';
import { RoleKind as GrpcRoleKind } from '@/generated/scylla/authz/v1/grant.ts';
import { ScopeKind } from '@/generated/scylla/authz/v1/permission.ts';
import type { GrantableRole } from '@/generated/scylla/authz/v1/grant.ts';
import { PermissionScope, RoleKind } from '@platform/authz';

const grantable = (overrides: Partial<GrantableRole> = {}): GrantableRole => ({
  roleId: { value: 'organization-admin' },
  scopeKind: ScopeKind.ORGANIZATION,
  kind: GrpcRoleKind.ADMIN,
  description: 'Full control of the organization',
  ...overrides,
});

describe('GrpcGrantableRoleMapper.toDomain', () => {
  it('unwraps the id, maps scope and kind, carries the description', () => {
    expect(GrpcGrantableRoleMapper.toDomain(grantable())).toEqual({
      roleId: 'organization-admin',
      scope: PermissionScope.ORGANIZATION,
      kind: RoleKind.ADMIN,
      description: 'Full control of the organization',
    });
  });

  it('defaults to an empty roleId when the wrapper is absent', () => {
    expect(GrpcGrantableRoleMapper.toDomain(grantable({ roleId: undefined })).roleId).toBe('');
  });

  it.each([
    [GrpcRoleKind.ADMIN, RoleKind.ADMIN],
    [GrpcRoleKind.AGENT, RoleKind.AGENT],
    [GrpcRoleKind.MEMBER, RoleKind.MEMBER],
    [GrpcRoleKind.UNSPECIFIED, RoleKind.UNSPECIFIED],
  ])('maps wire RoleKind %i to domain RoleKind %i', (wire, domain) => {
    expect(GrpcGrantableRoleMapper.toDomain(grantable({ kind: wire })).kind).toBe(domain);
  });
});
