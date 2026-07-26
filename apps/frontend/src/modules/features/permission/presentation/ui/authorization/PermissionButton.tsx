import type { ComponentProps, ReactNode } from 'react';
import { Trans } from '@lingui/react/macro';
import { Button } from '@shadcn';
import { Tooltip, TooltipContent, TooltipTrigger } from '@shadcn/tooltip.tsx';
import { cn } from '@shared/presentation/utils';
import type { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { PermissionTarget } from '@/modules/features/permission/domain/entities/effective-permissions.entity.ts';
import { useAuthorization } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

interface PermissionButtonProps extends ComponentProps<typeof Button> {
  /** The permission required to use this action. */
  permission: Permission;
  /** Resource the action is about; defaults to the current org/project context. */
  target?: PermissionTarget;
  /** Tooltip explaining why the button is disabled when the user lacks access. */
  deniedReason?: ReactNode;
}

/**
 * A {@link Button} that disables itself — with an explanatory tooltip — when the
 * current user lacks `permission`. The elegant middle ground between hiding an
 * action and letting the user click something that will only fail server-side.
 */
export const PermissionButton = ({
  permission,
  target,
  deniedReason,
  disabled,
  ...buttonProps
}: PermissionButtonProps) => {
  const { can } = useAuthorization();

  if (can(permission, target)) {
    return <Button disabled={disabled} {...buttonProps} />;
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        {/* Wrapper span so the tooltip still fires over the disabled button. */}
        <span className='inline-flex'>
          <Button
            {...buttonProps}
            disabled
            className={cn(buttonProps.className, 'pointer-events-none')}
          />
        </span>
      </TooltipTrigger>
      <TooltipContent>
        {deniedReason ?? <Trans>You don't have permission to do this.</Trans>}
      </TooltipContent>
    </Tooltip>
  );
};

export default PermissionButton;
