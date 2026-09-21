<script lang="ts">
  import { SvelteSet } from 'svelte/reactivity';
  import type { Permission } from '@platform/authz';
  import { checkedIdsOf, descendantIdsOf, findNode, type CheckboxNode } from './checkbox-tree.ts';
  import CheckboxTreeNode from './CheckboxTreeNode.svelte';

  interface Props {
    nodes: CheckboxNode[];
    /**
     * Ids checked on mount. Ids whose parent chain is not checked are dropped,
     * so the seeded state always matches what the tree can actually render.
     */
    checkedIds?: Permission[];
    onCheckedChange?: (checkedIds: Permission[]) => void;
    /** Disables every checkbox without altering the current selection. */
    allDisabled?: boolean;
  }

  let { nodes, checkedIds = [], onCheckedChange, allDisabled = false }: Props = $props();

  // Seeded once, then owned here. The React version re-seeded through an effect
  // comparing the incoming ids with its own state, because a parent feeding the
  // emitted selection straight back would otherwise wipe the selection being
  // made. The dialog recreates this component with `{#key}` when it opens for
  // another role, which is the same reset without the comparison.
  // svelte-ignore state_referenced_locally
  const checked = new SvelteSet(checkedIdsOf(nodes, new Set(checkedIds)));

  const toggle = (id: Permission, isChecked: boolean) => {
    const node = findNode(nodes, id);
    if (!node) return;

    if (isChecked) checked.add(id);
    // Unchecking cascades: a child is not conferred without its parent, so
    // leaving it ticked would show a selection the backend would not honour.
    else for (const descendant of descendantIdsOf(node)) checked.delete(descendant);

    onCheckedChange?.(checkedIdsOf(nodes, checked));
  };
</script>

<CheckboxTreeNode {nodes} {checked} disabled={allDisabled} {toggle} />
