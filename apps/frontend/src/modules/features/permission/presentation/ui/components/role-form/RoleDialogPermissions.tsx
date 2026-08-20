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
import { useMemo } from 'react';

interface RoleDialogPermissionsProps {
  scope: PermissionScope;
  permissions: Permission[];
  isPending: boolean;
  onPermissionsChange: (permissions: Permission[]) => void;
}

export const checkboxNodesFromPermissions = (
  permissionDefinitions: PermissionDefinition[],
): CheckboxNode<Permission>[] => {
  const nodeMap = new Map<string, CheckboxNode<Permission>>();
  const filteredDefinitions = permissionDefinitions.filter(def => !def.hidden);

  // 1. Initialisation des nœuds sans leurs enfants
  filteredDefinitions.forEach(def => {
    if (def.hidden) return;

    nodeMap.set(def.id.toString(), {
      id: def.id,
      label: def.label,
      children: [],
    });
  });

  const rootNodes: CheckboxNode<Permission>[] = [];

  // 2. Construction de la hiérarchie basée sur `dependsOn`
  filteredDefinitions.forEach(def => {
    const currentNode = nodeMap.get(def.id.toString())!;
    const parentIds = def.dependsOn?.map(p => p.toString()) ?? [];

    if (parentIds.length === 0) {
      rootNodes.push(currentNode);
    } else {
      parentIds.forEach(parentId => {
        const parentNode = nodeMap.get(parentId);
        if (parentNode) {
          parentNode.children!.push(currentNode);
        } else if (!rootNodes.includes(currentNode)) {
          // Si le parent n'est pas dans le tableau, le nœud est placé à la racine
          rootNodes.push(currentNode);
        }
      });
    }
  });

  // 3. Nettoyage des tableaux `children` vides pour éviter d'afficher le bouton "+"
  const cleanEmptyChildren = (nodes: CheckboxNode<Permission>[]) => {
    nodes.forEach(node => {
      if (node.children && node.children.length > 0) {
        cleanEmptyChildren(node.children);
      } else {
        delete node.children;
      }
    });
  };

  cleanEmptyChildren(rootNodes);

  return rootNodes;
};

export const RoleDialogPermissions = ({
  scope,
  permissions,
  isPending,
  onPermissionsChange,
}: RoleDialogPermissionsProps) => {
  const permissionsNodes = useMemo(() => {
    return checkboxNodesFromPermissions(getPermissionDefinitionsForScope(scope));
  }, [scope]);

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
            nodes={permissionsNodes}
            defaultCheckedIds={permissions}
            onCheckedChange={onPermissionsChange}
          />
        </div>
      </ScrollArea>
    </div>
  );
};
