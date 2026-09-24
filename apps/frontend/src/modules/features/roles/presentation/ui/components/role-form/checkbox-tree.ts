import type { Permission } from '@platform/authz';

export interface CheckboxNode {
  id: Permission;
  label: string;
  children?: CheckboxNode[];
}

/** The node and everything under it. */
export const descendantIdsOf = (node: CheckboxNode): Permission[] => [
  node.id,
  ...(node.children ?? []).flatMap(descendantIdsOf),
];

export const findNode = (
  nodes: readonly CheckboxNode[],
  id: Permission,
): CheckboxNode | null => {
  for (const node of nodes) {
    if (node.id === id) return node;
    const found = node.children ? findNode(node.children, id) : null;
    if (found) return found;
  }
  return null;
};

/** A node counts only when its box and its whole parent chain are checked. */
export const checkedIdsOf = (
  nodes: readonly CheckboxNode[],
  checked: ReadonlySet<Permission>,
): Permission[] => {
  const collected: Permission[] = [];

  const walk = (current: readonly CheckboxNode[]) => {
    for (const node of current) {
      if (!checked.has(node.id)) continue;
      collected.push(node.id);
      if (node.children) walk(node.children);
    }
  };

  walk(nodes);
  return collected;
};
