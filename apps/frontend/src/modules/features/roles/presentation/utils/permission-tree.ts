import type { Permission } from '@platform/authz';
import type { PermissionDefinition } from '@/modules/features/roles/presentation/utils/permission-mapping.ts';
import type { CheckboxNode } from '@/modules/features/roles/presentation/ui/components/role-form/checkbox-tree.ts';

/** Uses `dependsOn` as the parent; a parent outside `definitions` makes a root. */
export const buildPermissionTree = (
  definitions: PermissionDefinition[],
  label: (permission: Permission) => string,
): CheckboxNode[] => {
  const nodes = new Map<Permission, CheckboxNode>(
    definitions.map(definition => [definition.id, { id: definition.id, label: label(definition.id) }]),
  );

  const roots: CheckboxNode[] = [];

  for (const definition of definitions) {
    const node = nodes.get(definition.id)!;
    const parent = definition.dependsOn === undefined ? undefined : nodes.get(definition.dependsOn);

    if (!parent) {
      roots.push(node);
      continue;
    }
    parent.children = [...(parent.children ?? []), node];
  }

  return roots;
};
