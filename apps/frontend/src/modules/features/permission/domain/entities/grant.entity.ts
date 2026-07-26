import type {
  Permission,
  PermissionScope,
  PrincipalEntity,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

/**
 * What a grant confers. `unknown` means the backend sent a target arm newer
 * than this build — never read it as "grants nothing".
 */
export type GrantTarget =
  | { kind: 'role'; roleId: string }
  | { kind: 'permission'; permission: Permission }
  | { kind: 'unknown' };

/** The two targets this build is able to ask the backend to create. */
export type GrantTargetSpec = Exclude<GrantTarget, { kind: 'unknown' }>;

export interface GrantEntity {
  id: string;
  /** Who holds the grant: a user or an app. */
  principal: PrincipalEntity;
  /** A role, or a single direct permission. */
  target: GrantTarget;
  scope: PermissionScope;
  /** The org/project id the grant is bound to; empty for SYSTEM scope. */
  scopeId: string;
}
