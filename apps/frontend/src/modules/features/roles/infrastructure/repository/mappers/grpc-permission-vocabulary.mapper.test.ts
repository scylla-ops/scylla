import { describe, it, expect } from 'vitest';
import { GrpcPermissionVocabularyMapper } from './grpc-permission-vocabulary.mapper';
import { Permission as ProtoPermission, ScopeKind } from '@/generated/scylla/authz/v1/permission.ts';
import type { AuthzAction } from '@/generated/scylla/authz/v1/permission.ts';
import { Permission, PermissionScope } from '@platform/authz';

describe('GrpcPermissionVocabularyMapper.actionToDomain', () => {
  it('maps the permission and its minimum scope', () => {
    const action: AuthzAction = { permission: ProtoPermission.READ_PROJECT, minScope: ScopeKind.PROJECT };
    expect(GrpcPermissionVocabularyMapper.actionToDomain(action)).toEqual({
      permission: Permission.READ_PROJECT,
      minScope: PermissionScope.PROJECT,
    });
  });
});

describe('GrpcPermissionVocabularyMapper.toDomain', () => {
  it('maps every action in the vocabulary, in order', () => {
    const actions: AuthzAction[] = [
      { permission: ProtoPermission.CREATE_ORGANIZATION, minScope: ScopeKind.SYSTEM },
      { permission: ProtoPermission.READ_PROJECT, minScope: ScopeKind.PROJECT },
    ];
    expect(GrpcPermissionVocabularyMapper.toDomain(actions)).toEqual({
      actions: [
        { permission: Permission.CREATE_ORGANIZATION, minScope: PermissionScope.SYSTEM },
        { permission: Permission.READ_PROJECT, minScope: PermissionScope.PROJECT },
      ],
    });
  });

  it('an empty vocabulary maps to an empty action list', () => {
    expect(GrpcPermissionVocabularyMapper.toDomain([])).toEqual({ actions: [] });
  });
});
