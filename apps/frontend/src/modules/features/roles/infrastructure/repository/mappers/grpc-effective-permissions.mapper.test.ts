import { describe, it, expect } from 'vitest';
import { GrpcEffectivePermissionsMapper } from './grpc-effective-permissions.mapper';
import type { EffectiveScope } from '@/generated/scylla/authz/v1/role.ts';
import { PermissionScope } from '@platform/authz';

describe('GrpcEffectivePermissionsMapper.scopeToDomain', () => {
  it('flattens the scope ref and maps the access', () => {
    const scope: EffectiveScope = {
      scope: { scope: { oneofKind: 'organization', organization: { organizationId: { value: 'org-1' } } } },
      access: { access: { oneofKind: 'fullControl', fullControl: {} } },
    };
    expect(GrpcEffectivePermissionsMapper.scopeToDomain(scope)).toEqual({
      scope: PermissionScope.ORGANIZATION,
      scopeId: 'org-1',
      access: { kind: 'fullControl' },
    });
  });

  it('an absent scope ref reads as UNSPECIFIED', () => {
    const scope: EffectiveScope = { scope: undefined, access: { access: { oneofKind: 'fullControl', fullControl: {} } } };
    expect(GrpcEffectivePermissionsMapper.scopeToDomain(scope).scope).toBe(PermissionScope.UNSPECIFIED);
  });

  it('an absent access reads as unknown', () => {
    const scope: EffectiveScope = {
      scope: { scope: { oneofKind: 'system', system: {} } },
      access: undefined,
    };
    expect(GrpcEffectivePermissionsMapper.scopeToDomain(scope).access).toEqual({ kind: 'unknown' });
  });
});

describe('GrpcEffectivePermissionsMapper.toDomain', () => {
  it('maps every scope, in order', () => {
    const scopes: EffectiveScope[] = [
      { scope: { scope: { oneofKind: 'system', system: {} } }, access: { access: { oneofKind: 'fullControl', fullControl: {} } } },
      {
        scope: { scope: { oneofKind: 'project', project: { projectId: { value: 'project-1' } } } },
        access: { access: { oneofKind: 'restricted', restricted: { permissions: [] } } },
      },
    ];
    const domain = GrpcEffectivePermissionsMapper.toDomain(scopes);
    expect(domain.scopes).toHaveLength(2);
    expect(domain.scopes[0]).toEqual({ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } });
    expect(domain.scopes[1]).toEqual({
      scope: PermissionScope.PROJECT,
      scopeId: 'project-1',
      access: { kind: 'restricted', permissions: [] },
    });
  });

  it('no scopes maps to an empty list', () => {
    expect(GrpcEffectivePermissionsMapper.toDomain([])).toEqual({ scopes: [] });
  });
});
