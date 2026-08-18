import { Badge, Label } from '@shadcn';
import { Trans } from '@lingui/react/macro';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import { getPermissionDefinitionsForScope } from '@/modules/features/permission/presentation/utils/permission-mapping.ts';
import { Checkbox } from '@shadcn/checkbox.tsx';
import {
  type Permission,
  type PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';

interface RoleDialogPermissionsProps {
  scope: PermissionScope;
  permissions: Set<Permission>;
  isPending: boolean;
  togglePermission: (permission: Permission) => void;
}

export const RoleDialogPermissions = ({
  scope,
  permissions,
  isPending,
  togglePermission,
}: RoleDialogPermissionsProps) => {
  return (
    <div className='flex flex-col gap-1.5'>
      <div className='flex items-center justify-between'>
        <Label>
          <Trans>Permissions</Trans>
        </Label>
        <Badge variant='secondary'>
          <Trans>{permissions.size} selected</Trans>
        </Badge>
      </div>
      <ScrollArea className='h-56 rounded-lg border border-slate-200 p-2'>
        <div className='flex flex-col gap-0.5'>
          {getPermissionDefinitionsForScope(scope)?.map(permission => (
            <label
              key={permission.id}
              className='flex items-center gap-2 rounded-md px-2 py-1.5 cursor-pointer hover:bg-slate-50'
            >
              <Checkbox
                checked={permissions.has(permission.id)}
                disabled={isPending}
                onCheckedChange={() => togglePermission(permission.id)}
              />
              <span className='text-sm capitalize'>{permission.label}</span>
            </label>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
};
