import { Badge, Label } from '@shadcn';
import { Trans } from '@lingui/react/macro';
import { ScrollArea } from '@shadcn/scroll-area.tsx';
import {
  getPermissionDefinitionsForScope,
  type PermissionDefinition,
} from '@/modules/features/permission/presentation/utils/permission-mapping.ts';
import {
  type Permission,
  type PermissionScope,
} from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { type CheckboxNode, CheckboxTree } from '@shared/presentation/ui/forms/CheckboxTree.tsx';
import { useRef } from 'react';

interface RoleDialogPermissionsProps {
  scope: PermissionScope;
  permissions: Set<Permission>;
  isPending: boolean;
  togglePermission: (permission: Permission) => void;
}

const nodesMock: CheckboxNode[] = [
  {
    id: '1',
    label: 'Node 1',
    children: [
      { id: '2', label: 'Node 2', children: [] },
      {
        id: '2',
        label: 'Node 2',
        children: [
          { id: '2', label: 'Node 2', children: [] },
          { id: '2', label: 'Node 2', children: [{ id: '2', label: 'Node 2', children: [] }] },
          { id: '2', label: 'Node 2', children: [{ id: '2', label: 'Node 2', children: [] }] },
        ],
      },
    ],
  },
  { id: '3', label: 'Node 3', children: [] },
];

const checkboxNodesFromPermissions = (
  permissionDefinitions: PermissionDefinition[],
): CheckboxNode[] => {
  return permissionDefinitions.map(def => ({
    id: def.id.toString(),
    label: def.label,
    children: [
      { id: 'children', label: 'children', children: [{ id: 'children', label: 'children' }] },
    ],
  }));
};

export const RoleDialogPermissions = ({
  scope,
  permissions,
  isPending,
  togglePermission,
}: RoleDialogPermissionsProps) => {
  const permissionsNodes = useRef(
    checkboxNodesFromPermissions(getPermissionDefinitionsForScope(scope)),
  );

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
          <CheckboxTree nodes={permissionsNodes.current} />
        </div>
      </ScrollArea>
    </div>
  );
};
