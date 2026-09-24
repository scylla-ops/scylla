// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { Permission } from '@platform/authz';
import { checkedIdsOf, descendantIdsOf, findNode, type CheckboxNode } from '../checkbox-tree.ts';

/**
 * The permission tree's rules, without a DOM.
 *
 * They came out of `shared/presentation/ui/forms/CheckboxTree.tsx`, where they
 * were entangled with the widget. Pinning them here is what lets the component
 * test stay about clicking.
 */
const tree: CheckboxNode[] = [
  {
    id: Permission.READ_PIPELINE,
    label: 'Read pipeline',
    children: [
      { id: Permission.RUN_PIPELINE, label: 'Run pipeline' },
      {
        id: Permission.LIST_JOBS,
        label: 'List jobs',
        children: [{ id: Permission.READ_JOB_LOGS, label: 'Read job logs' }],
      },
    ],
  },
  { id: Permission.CREATE_SECRET, label: 'Create secret' },
];

describe('descendantIdsOf', () => {
  it('includes the node itself, so unchecking clears the box that was clicked', () => {
    expect(descendantIdsOf(tree[1])).toEqual([Permission.CREATE_SECRET]);
  });

  it('reaches grandchildren, not just the first level', () => {
    expect(descendantIdsOf(tree[0])).toEqual([
      Permission.READ_PIPELINE,
      Permission.RUN_PIPELINE,
      Permission.LIST_JOBS,
      Permission.READ_JOB_LOGS,
    ]);
  });
});

describe('findNode', () => {
  it('finds a node nested two levels down', () => {
    expect(findNode(tree, Permission.READ_JOB_LOGS)?.label).toBe('Read job logs');
  });

  it('answers null for a permission the tree does not carry', () => {
    expect(findNode(tree, Permission.MANAGE_ROLES)).toBeNull();
  });
});

describe('checkedIdsOf', () => {
  it('drops a child whose parent is unchecked — it is not conferred', () => {
    const checked = new Set([Permission.RUN_PIPELINE, Permission.CREATE_SECRET]);

    expect(checkedIdsOf(tree, checked)).toEqual([Permission.CREATE_SECRET]);
  });

  it('keeps a child once its whole parent chain is checked', () => {
    const checked = new Set([
      Permission.READ_PIPELINE,
      Permission.LIST_JOBS,
      Permission.READ_JOB_LOGS,
    ]);

    expect(checkedIdsOf(tree, checked)).toEqual([
      Permission.READ_PIPELINE,
      Permission.LIST_JOBS,
      Permission.READ_JOB_LOGS,
    ]);
  });

  it('stops at the first broken link of the chain', () => {
    // The grandchild is ticked, its parent is not: neither reads as checked.
    const checked = new Set([Permission.READ_PIPELINE, Permission.READ_JOB_LOGS]);

    expect(checkedIdsOf(tree, checked)).toEqual([Permission.READ_PIPELINE]);
  });

  it('returns ids in render order, not in the order they were ticked', () => {
    const checked = new Set([
      Permission.CREATE_SECRET,
      Permission.READ_PIPELINE,
      Permission.RUN_PIPELINE,
    ]);

    expect(checkedIdsOf(tree, checked)).toEqual([
      Permission.READ_PIPELINE,
      Permission.RUN_PIPELINE,
      Permission.CREATE_SECRET,
    ]);
  });
});
