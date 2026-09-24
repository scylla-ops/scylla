<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import type { Permission } from '@platform/authz';
  import { checkedIdsOf, descendantIdsOf, findNode, type CheckboxNode } from './checkbox-tree.ts';
  import CheckboxTreeNode from './CheckboxTreeNode.svelte';

  interface Props {
    nodes: CheckboxNode[];
    /** Ids whose parent is not checked are dropped. */
    checkedIds?: Permission[];
    onCheckedChange?: (checkedIds: Permission[]) => void;
    allDisabled?: boolean;
  }

  let { nodes, checkedIds = [], onCheckedChange, allDisabled = false }: Props = $props();

  // Seeded once, then owned here: the dialog rebuilds it for another role.
  // svelte-ignore state_referenced_locally
  const checked = new SvelteSet(checkedIdsOf(nodes, new Set(checkedIds)));

  const toggle = (id: Permission, isChecked: boolean) => {
    const node = findNode(nodes, id);
    if (!node) return;

    if (isChecked) checked.add(id);
    // A child is not conferred without its parent.
    else for (const descendant of descendantIdsOf(node)) checked.delete(descendant);

    onCheckedChange?.(checkedIdsOf(nodes, checked));
  };
</script>

<CheckboxTreeNode {nodes} {checked} disabled={allDisabled} {toggle} />
