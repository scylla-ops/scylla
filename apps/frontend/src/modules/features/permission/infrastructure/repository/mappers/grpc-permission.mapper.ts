import type {
  Access,
  Permission,
  PrincipalRef,
  ScopeKind,
  ScopeRef,
} from '@/generated/scylla/authz/v1/permission.ts';
import {
  type AccessEntity,
  type AccessSpec,
  type Permission as PermissionDomain,
  PermissionScope,
  type PrincipalEntity,
  PrincipalKind,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

/** A scope as the domain carries it: a kind plus the id it is bound to. */
export interface ScopeBinding {
  scope: PermissionScope;
  /** Empty for SYSTEM scope, and for a scope arm this build does not know. */
  scopeId: string;
}

export class GrpcPermissionMapper {
  public static toDomain(permission: Permission): PermissionDomain {
    return permission;
  }

  public static toGrpc(permission: PermissionDomain): Permission {
    return permission;
  }

  // ── Scope ───────────────────────────────────────────────────────────────────
  // `ScopeKind` (id-free) is still used for catalog filters; `ScopeRef` is the
  // id-carrying form used by grants and effective scopes.

  public static scopeToDomain(scope: ScopeKind): PermissionScope {
    return scope as unknown as PermissionScope;
  }

  public static scopeToGrpc(scope: PermissionScope): ScopeKind {
    return scope as unknown as ScopeKind;
  }

  /**
   * Flattens a `ScopeRef` into the domain's (kind, id) pair.
   * An arm this build does not know about surfaces as `UNSPECIFIED` rather
   * than being silently read as SYSTEM.
   */
  public static scopeRefToDomain(ref: ScopeRef | undefined): ScopeBinding {
    switch (ref?.scope.oneofKind) {
      case 'system':
        return { scope: PermissionScope.SYSTEM, scopeId: '' };
      case 'organization':
        return {
          scope: PermissionScope.ORGANIZATION,
          scopeId: ref.scope.organization.organizationId?.value ?? '',
        };
      case 'project':
        return {
          scope: PermissionScope.PROJECT,
          scopeId: ref.scope.project.projectId?.value ?? '',
        };
      default:
        return { scope: PermissionScope.UNSPECIFIED, scopeId: '' };
    }
  }

  /** Builds the `ScopeRef` for a scope the user picked. Throws on UNSPECIFIED. */
  public static scopeRefToGrpc(scope: PermissionScope, scopeId: string): ScopeRef {
    switch (scope) {
      case PermissionScope.SYSTEM:
        return { scope: { oneofKind: 'system', system: {} } };
      case PermissionScope.ORGANIZATION:
        return {
          scope: {
            oneofKind: 'organization',
            organization: { organizationId: { value: scopeId } },
          },
        };
      case PermissionScope.PROJECT:
        return { scope: { oneofKind: 'project', project: { projectId: { value: scopeId } } } };
      default:
        throw new Error('A scope must be chosen before sending it to the backend.');
    }
  }

  // ── Principal ───────────────────────────────────────────────────────────────

  /** A grant can target a user or an app. An unknown arm stays UNSPECIFIED. */
  public static principalRefToDomain(ref: PrincipalRef | undefined): PrincipalEntity {
    switch (ref?.principal.oneofKind) {
      case 'user':
        return { kind: PrincipalKind.USER, id: ref.principal.user.userId?.value ?? '' };
      case 'app':
        return { kind: PrincipalKind.APP, id: ref.principal.app.appId?.value ?? '' };
      default:
        return { kind: PrincipalKind.UNSPECIFIED, id: '' };
    }
  }

  public static principalRefToGrpc(principal: PrincipalEntity): PrincipalRef {
    switch (principal.kind) {
      case PrincipalKind.USER:
        return { principal: { oneofKind: 'user', user: { userId: { value: principal.id } } } };
      case PrincipalKind.APP:
        return { principal: { oneofKind: 'app', app: { appId: { value: principal.id } } } };
      default:
        throw new Error('A principal must be chosen before sending it to the backend.');
    }
  }

  // ── Access ──────────────────────────────────────────────────────────────────

  /**
   * `Access` is a oneof: full control, or an explicit permission set.
   * A missing message or an unknown arm surfaces as `unknown` — never as an
   * empty permission list, which would read as "holds nothing".
   */
  public static accessToDomain(access: Access | undefined): AccessEntity {
    switch (access?.access.oneofKind) {
      case 'fullControl':
        return { kind: 'fullControl' };
      case 'restricted':
        return {
          kind: 'restricted',
          permissions: access.access.restricted.permissions.map(GrpcPermissionMapper.toDomain),
        };
      default:
        return { kind: 'unknown' };
    }
  }

  public static accessToGrpc(access: AccessSpec): Access {
    return access.kind === 'fullControl'
      ? { access: { oneofKind: 'fullControl', fullControl: {} } }
      : {
          access: {
            oneofKind: 'restricted',
            restricted: { permissions: access.permissions.map(GrpcPermissionMapper.toGrpc) },
          },
        };
  }
}
