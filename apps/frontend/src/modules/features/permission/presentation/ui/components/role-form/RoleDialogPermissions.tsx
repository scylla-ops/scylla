import { useMemo } from 'react';
import { Badge, Label } from '@shadcn';
import { Trans } from '@lingui/react/macro';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { CheckboxTree } from '@shared/presentation/ui/forms/CheckboxTree.tsx';
import type {
  Permission,
  PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { getPermissionDefinitionsForScope } from '@/modules/features/permission/presentation/utils/permission-mapping.ts';
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

  const permissionNodes = useMemo(
    () => buildPermissionTree(getPermissionDefinitionsForScope(scope), permissionLabel),
    [scope, permissionLabel],
  );

  return (
    <div className='flex flex-col gap-1.5'>
      <div className='flex items-center justify-between'>
        <Label>
          <Trans>Permissions</Trans>
        </Label>
        <Badge variant='secondary'>
          <Trans>{permissions.length} selected</Trans>
        </Badge>
      </div>
      <ScrollArea className='h-56 rounded-lg border border-slate-200 p-2'>
        <div className='flex flex-col gap-0.5'>
          <CheckboxTree
            allDisabled={isPending}
            nodes={permissionNodes}
            defaultCheckedIds={permissions}
            onCheckedChange={onPermissionsChange}
          />
        </div>
      </ScrollArea>
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
