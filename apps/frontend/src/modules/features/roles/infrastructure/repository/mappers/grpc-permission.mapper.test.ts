import { describe, it, expect } from 'vitest';
import { GrpcPermissionMapper } from './grpc-permission.mapper';
import { Permission as ProtoPermission } from '@/generated/scylla/authz/v1/permission.ts';
import type { ScopeRef, PrincipalRef, Access } from '@/generated/scylla/authz/v1/permission.ts';
import { Permission, PermissionScope, PrincipalKind } from '@platform/authz';

describe('GrpcPermissionMapper.toDomain / toGrpc (Permission)', () => {
  it('is a value-identical pass-through in both directions', () => {
    expect(GrpcPermissionMapper.toDomain(ProtoPermission.READ_PROJECT)).toBe(Permission.READ_PROJECT);
    expect(GrpcPermissionMapper.toGrpc(Permission.READ_PROJECT)).toBe(ProtoPermission.READ_PROJECT);
  });
});

describe('GrpcPermissionMapper.scopeRefToDomain', () => {
  it('an absent ref reads as UNSPECIFIED', () => {
    expect(GrpcPermissionMapper.scopeRefToDomain(undefined)).toEqual({
      scope: PermissionScope.UNSPECIFIED,
      scopeId: '',
    });
  });

  it('system has no id', () => {
    const ref: ScopeRef = { scope: { oneofKind: 'system', system: {} } };
    expect(GrpcPermissionMapper.scopeRefToDomain(ref)).toEqual({
      scope: PermissionScope.SYSTEM,
      scopeId: '',
    });
  });

  it('organization carries its organizationId', () => {
    const ref: ScopeRef = {
      scope: { oneofKind: 'organization', organization: { organizationId: { value: 'org-1' } } },
    };
    expect(GrpcPermissionMapper.scopeRefToDomain(ref)).toEqual({
      scope: PermissionScope.ORGANIZATION,
      scopeId: 'org-1',
    });
  });

  it('an organization ref with no id defaults to an empty scopeId, not a crash', () => {
    const ref: ScopeRef = { scope: { oneofKind: 'organization', organization: {} } };
    expect(GrpcPermissionMapper.scopeRefToDomain(ref).scopeId).toBe('');
  });

  it('project carries its projectId', () => {
    const ref: ScopeRef = { scope: { oneofKind: 'project', project: { projectId: { value: 'project-1' } } } };
    expect(GrpcPermissionMapper.scopeRefToDomain(ref)).toEqual({
      scope: PermissionScope.PROJECT,
      scopeId: 'project-1',
    });
  });

  it('a scope oneof arm this build does not know reads as UNSPECIFIED rather than SYSTEM', () => {
    const ref: ScopeRef = { scope: { oneofKind: undefined } };
    expect(GrpcPermissionMapper.scopeRefToDomain(ref)).toEqual({
      scope: PermissionScope.UNSPECIFIED,
      scopeId: '',
    });
  });
});

describe('GrpcPermissionMapper.scopeRefToGrpc', () => {
  it('builds a system ref', () => {
    expect(GrpcPermissionMapper.scopeRefToGrpc(PermissionScope.SYSTEM, '')).toEqual({
      scope: { oneofKind: 'system', system: {} },
    });
  });

  it('builds an organization ref', () => {
    expect(GrpcPermissionMapper.scopeRefToGrpc(PermissionScope.ORGANIZATION, 'org-1')).toEqual({
      scope: { oneofKind: 'organization', organization: { organizationId: { value: 'org-1' } } },
    });
  });

  it('builds a project ref', () => {
    expect(GrpcPermissionMapper.scopeRefToGrpc(PermissionScope.PROJECT, 'project-1')).toEqual({
      scope: { oneofKind: 'project', project: { projectId: { value: 'project-1' } } },
    });
  });

  it('refuses to send an UNSPECIFIED scope - the backend rejects it anyway', () => {
    expect(() => GrpcPermissionMapper.scopeRefToGrpc(PermissionScope.UNSPECIFIED, '')).toThrow(
      /scope must be chosen/,
    );
  });
});

describe('GrpcPermissionMapper.principalRefToDomain', () => {
  it('an absent ref reads as UNSPECIFIED', () => {
    expect(GrpcPermissionMapper.principalRefToDomain(undefined)).toEqual({
      kind: PrincipalKind.UNSPECIFIED,
      id: '',
    });
  });

  it('maps a user principal', () => {
    const ref: PrincipalRef = { principal: { oneofKind: 'user', user: { userId: { value: 'user-1' } } } };
    expect(GrpcPermissionMapper.principalRefToDomain(ref)).toEqual({ kind: PrincipalKind.USER, id: 'user-1' });
  });

  it('maps an app principal', () => {
    const ref: PrincipalRef = { principal: { oneofKind: 'app', app: { appId: { value: 'app-1' } } } };
    expect(GrpcPermissionMapper.principalRefToDomain(ref)).toEqual({ kind: PrincipalKind.APP, id: 'app-1' });
  });

  it('a principal oneof arm this build does not know reads as UNSPECIFIED', () => {
    const ref: PrincipalRef = { principal: { oneofKind: undefined } };
    expect(GrpcPermissionMapper.principalRefToDomain(ref)).toEqual({ kind: PrincipalKind.UNSPECIFIED, id: '' });
  });
});

describe('GrpcPermissionMapper.principalRefToGrpc', () => {
  it('builds a user ref', () => {
    expect(GrpcPermissionMapper.principalRefToGrpc({ kind: PrincipalKind.USER, id: 'user-1' })).toEqual({
      principal: { oneofKind: 'user', user: { userId: { value: 'user-1' } } },
    });
  });

  it('builds an app ref', () => {
    expect(GrpcPermissionMapper.principalRefToGrpc({ kind: PrincipalKind.APP, id: 'app-1' })).toEqual({
      principal: { oneofKind: 'app', app: { appId: { value: 'app-1' } } },
    });
  });

  it('refuses to send an UNSPECIFIED principal', () => {
    expect(() =>
      GrpcPermissionMapper.principalRefToGrpc({ kind: PrincipalKind.UNSPECIFIED, id: '' }),
    ).toThrow(/principal must be chosen/);
  });
});

describe('GrpcPermissionMapper.accessToDomain', () => {
  it('an absent access reads as unknown, never as an empty permission set', () => {
    expect(GrpcPermissionMapper.accessToDomain(undefined)).toEqual({ kind: 'unknown' });
  });

  it('maps fullControl', () => {
    const access: Access = { access: { oneofKind: 'fullControl', fullControl: {} } };
    expect(GrpcPermissionMapper.accessToDomain(access)).toEqual({ kind: 'fullControl' });
  });

  it('maps a restricted permission set', () => {
    const access: Access = {
      access: {
        oneofKind: 'restricted',
        restricted: { permissions: [ProtoPermission.READ_PROJECT, ProtoPermission.UPDATE_PROJECT] },
      },
    };
    expect(GrpcPermissionMapper.accessToDomain(access)).toEqual({
      kind: 'restricted',
      permissions: [Permission.READ_PROJECT, Permission.UPDATE_PROJECT],
    });
  });

  it('an access oneof arm this build does not know reads as unknown', () => {
    const access: Access = { access: { oneofKind: undefined } };
    expect(GrpcPermissionMapper.accessToDomain(access)).toEqual({ kind: 'unknown' });
  });
});

describe('GrpcPermissionMapper.accessToGrpc', () => {
  it('builds a fullControl access', () => {
    expect(GrpcPermissionMapper.accessToGrpc({ kind: 'fullControl' })).toEqual({
      access: { oneofKind: 'fullControl', fullControl: {} },
    });
  });

  it('builds a restricted access from an explicit permission list', () => {
    expect(
      GrpcPermissionMapper.accessToGrpc({ kind: 'restricted', permissions: [Permission.READ_PROJECT] }),
    ).toEqual({
      access: { oneofKind: 'restricted', restricted: { permissions: [ProtoPermission.READ_PROJECT] } },
    });
  });
});
