import { useCallback, useMemo } from 'react';
import { Badge, Label } from '@shadcn';
import { Trans } from '@lingui/react/macro';
import { Checkbox } from '@shadcn/checkbox.tsx';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { CheckboxTree } from '@shared/presentation/ui/forms/CheckboxTree.tsx';
import type {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import {
  getAlwaysGrantedPermissionsForScope,
  getEditablePermissionDefinitionsForScope,
  withImplicitPermissions,
} from '@/modules/features/permission/presentation/utils/permission-mapping.ts';
import { buildPermissionTree } from '@/modules/features/permission/presentation/utils/permission-tree.ts';
import { usePermissionLabels } from '@/modules/features/permission/presentation/hooks/use-permission-labels.ts';

interface RoleDialogPermissionsProps {
  scope: PermissionScope;
  permissions: Permission[];
  /**
   * How many permissions the role holds outside this build's catalog. They are
   * kept on save; the count is shown so nobody thinks they vanished.
   */
  preservedCount: number;
  isPending: boolean;
  onPermissionsChange: (permissions: Permission[]) => void;
}

export const RoleDialogPermissions = ({
  scope,
  permissions,
  preservedCount,
  isPending,
  onPermissionsChange,
}: RoleDialogPermissionsProps) => {
  const { permissionLabel } = usePermissionLabels();

  // Labels read against the *role's* scope, so a project permission conferred by
  // an organization role says "every project" rather than "the project".
  const labelForScope = useCallback(
    (permission: Permission) => permissionLabel(permission, scope),
    [permissionLabel, scope],
  );

  const permissionNodes = useMemo(
    () => buildPermissionTree(getEditablePermissionDefinitionsForScope(scope), labelForScope),
    [scope, labelForScope],
  );

  /**
   * Conferred by construction at this scope, so shown ticked and locked rather
   * than hidden: the reader still learns the role carries it, and nobody can
   * build a role that admits someone to a place they cannot see.
   */
  const alwaysGranted = useMemo(() => getAlwaysGrantedPermissionsForScope(scope), [scope]);

  /** What will actually be written — riders included. The honest count. */
  const conferredCount = useMemo(
    () => withImplicitPermissions(scope, permissions).length,
    [scope, permissions],
  );

  return (
    <div className='flex flex-col gap-1.5'>
      <div className='flex items-center justify-between'>
        <Label>
          <Trans>Permissions</Trans>
        </Label>
        <Badge variant='secondary'>
          <Trans>{conferredCount} selected</Trans>
        </Badge>
      </div>
      <ScrollArea className='h-56 rounded-lg border p-2'>
        <div className='flex flex-col gap-0.5'>
          {alwaysGranted.map(permission => (
            <label
              key={permission}
              className='flex items-center gap-2 rounded-md px-2 py-1.5 cursor-not-allowed'
              aria-disabled
            >
              <Checkbox checked disabled />
              <span className='text-sm'>{labelForScope(permission)}</span>
              <Badge variant='outline' className='ml-auto text-[10px]'>
                <Trans>Always</Trans>
              </Badge>
            </label>
          ))}
          <CheckboxTree
            allDisabled={isPending}
            nodes={permissionNodes}
            defaultCheckedIds={permissions}
            onCheckedChange={onPermissionsChange}
          />
        </div>
      </ScrollArea>
      {alwaysGranted.length > 0 && (
        <p className='text-xs text-muted-foreground'>
          <Trans>
            Holding a role in an organization is what belonging to it means, so every organization
            role carries it. An organization role applies to every project of the organization.
          </Trans>
        </p>
      )}
      {preservedCount > 0 && (
        <p className='text-xs text-muted-foreground'>
          <Trans>
            This role also holds {preservedCount} permission(s) not managed here. They are kept
            unchanged.
          </Trans>
        </p>
      )}
    </div>
  );
};
