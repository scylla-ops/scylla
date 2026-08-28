import type { ReactNode } from 'react';
import { Loader2 } from 'lucide-react';
import type { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { PermissionTarget } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import { useAuthorization } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';
import { PermissionDenied } from '@/modules/features/permission/presentation/ui/authorization/PermissionDenied.tsx';

interface RequirePermissionProps {
  /** The permission required to see this content. */
  permission: Permission;
  /** Resource the check is about; defaults to the current org/project context. */
  target?: PermissionTarget;
  /** Custom denial message shown in the {@link PermissionDenied} panel. */
  message?: ReactNode;
  children: ReactNode;
}

/**
 * Page/section-level gate: renders `children` when the current user holds
 * `permission`, and a full {@link PermissionDenied} panel otherwise. While the
 * permission lookup is still in flight it shows a quiet spinner — never the
 * gated content, and never a premature denial. Unlike {@link Can} (which hides),
 * this *explains* — use it for whole pages or large sections, e.g. wrapped
 * around a route element, so a direct URL hit still gets a clear answer.
 */
export const RequirePermission = ({
  permission,
  target,
  message,
  children,
}: RequirePermissionProps) => {
  const { can, ready } = useAuthorization();

  if (!ready) {
    return (
      <div className='flex h-full w-full items-center justify-center py-16'>
        <Loader2 className='size-6 animate-spin text-muted-foreground' />
      </div>
    );
  }

  return can(permission, target) ? <>{children}</> : <PermissionDenied message={message} />;
};

export default RequirePermission;
