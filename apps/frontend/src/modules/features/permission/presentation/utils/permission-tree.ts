import type { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import type { PermissionDefinition } from '@/modules/features/permission/presentation/utils/permission-mapping.ts';
import type { CheckboxNode } from '@shared/presentation/ui/forms/CheckboxTree.tsx';

/**
 * Turns catalog entries into the tree the role editor renders, using each
 * entry's `dependsOn` as its parent. A definition whose parent isn't part of
 * `definitions` — the usual case when the list is filtered down to one scope —
 * becomes a root.
 *
 * Every definition appears exactly once: `dependsOn` is a single parent, so no
 * node can be reachable through two branches (which would duplicate its
 * checkbox id in the DOM). Catalog order is preserved among siblings.
 */
export const buildPermissionTree = (
  definitions: PermissionDefinition[],
  label: (permission: Permission) => string,
): CheckboxNode<Permission>[] => {
  const nodes = new Map<Permission, CheckboxNode<Permission>>(
    definitions.map(definition => [definition.id, { id: definition.id, label: label(definition.id) }]),
  );

  const roots: CheckboxNode<Permission>[] = [];

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
