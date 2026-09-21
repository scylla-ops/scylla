import type { Permission } from '@platform/authz';

/**
 * The permission tree's shape and the pure rules over it.
 *
 * It lived in `shared/presentation/ui/forms/CheckboxTree.tsx` while React owned
 * it, on the strength of being "generic". It never had a second consumer: the
 * role editor is the only screen in the app with a tree of checkboxes, and the
 * rules below — a child counts only when its whole parent chain is checked,
 * unchecking a parent clears its descendants — are the permission model's, not
 * a widget's. So the port lands here, in the feature that needs it. If a second
 * consumer ever appears, `shared/` is one move away — and it can take the
 * generic back then. The React original was generic over `string | number` and
 * only ever instantiated with `Permission`; typing it concretely here is what
 * lets the component drop the `generics` attribute, which `tsc` and
 * `svelte-check` disagree about (see `refacto_svelte.md`, end of Phase 1).
 */

export interface CheckboxNode {
  id: Permission;
  label: string;
  children?: CheckboxNode[];
}

/** The node and everything under it — what unchecking has to clear. */
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

/**
 * The ids that actually read as checked, in render order.
 *
 * A node counts only when its own box **and** its whole parent chain are
 * checked: a permission whose parent is off is not conferred, and showing it as
 * selected would promise access the backend will refuse.
 */
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
