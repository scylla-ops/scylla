import type { ReactNode } from 'react';
import type { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { PermissionTarget } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import { useAuthorization } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

interface CanProps {
  /** The permission the current user must hold to see `children`. */
  permission: Permission;
  /** Resource the check is about; defaults to the current org/project context. */
  target?: PermissionTarget;
  /** Rendered instead of `children` when the user lacks the permission. */
  fallback?: ReactNode;
  children: ReactNode;
}

/**
 * Renders `children` only when the current user may perform `permission` on
 * `target`. Use it to hide UI a user has no access to (menu entries, cards, …).
 */
export const Can = ({ permission, target, fallback = null, children }: CanProps) => {
  const { can } = useAuthorization();
  return <>{can(permission, target) ? children : fallback}</>;
};

export default Can;
