import { type Permission, Scope } from '@/generated/permission.ts';
import {
  type Permission as PermissionDomain,
  PermissionScope,
} from '@/modules/features/permission/domain/models/permission.model.ts';

export class GrpcPermissionMapper {
  public static toDomain(permission: Permission): PermissionDomain {
    return permission as unknown as PermissionDomain;
  }

  public static toGrpc(permission: PermissionDomain): Permission {
    return permission as unknown as Permission;
  }

  public static scopeToDomain(scope: Scope): PermissionScope {
    return scope as unknown as PermissionScope;
  }

  public static scopeToGrpc(scope: PermissionScope): Scope {
    return scope as unknown as Scope;
  }
}
