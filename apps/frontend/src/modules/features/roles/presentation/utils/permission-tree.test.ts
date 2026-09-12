import { describe, it, expect } from 'vitest';
import { Permission, PermissionScope } from '@platform/authz';
import { buildPermissionTree } from './permission-tree';
import type { PermissionDefinition } from './permission-mapping.ts';

const def = (overrides: Partial<PermissionDefinition> & { id: Permission }): PermissionDefinition => ({
  label: { id: Permission[overrides.id], message: Permission[overrides.id] },
  scope: PermissionScope.PROJECT,
  ...overrides,
});

// A numeric enum's reverse mapping gives back its member name - a readable,
// unique stand-in label for these tests (the real labeler resolves a
// MessageDescriptor instead, already covered by the last test below).
const label = (permission: Permission): string => Permission[permission];
const named = (permission: Permission) => Permission[permission];

describe('buildPermissionTree', () => {
  it('a definition with no dependsOn is a root', () => {
    const tree = buildPermissionTree([def({ id: Permission.READ_PROJECT })], label);
    expect(tree).toEqual([{ id: Permission.READ_PROJECT, label: named(Permission.READ_PROJECT) }]);
  });

  it('nests a definition under its dependsOn parent, when the parent is present in the list', () => {
    const tree = buildPermissionTree(
      [
        def({ id: Permission.READ_PROJECT }),
        def({ id: Permission.UPDATE_PROJECT, dependsOn: Permission.READ_PROJECT }),
      ],
      label,
    );

    expect(tree).toEqual([
      {
        id: Permission.READ_PROJECT,
        label: named(Permission.READ_PROJECT),
        children: [{ id: Permission.UPDATE_PROJECT, label: named(Permission.UPDATE_PROJECT) }],
      },
    ]);
  });

  it('a dependsOn whose target is absent from the list becomes a root instead (the usual case for a single-scope filter)', () => {
    // READ_PROJECT is not in `definitions` - only its dependent is.
    const tree = buildPermissionTree(
      [def({ id: Permission.UPDATE_PROJECT, dependsOn: Permission.READ_PROJECT })],
      label,
    );

    expect(tree).toEqual([{ id: Permission.UPDATE_PROJECT, label: named(Permission.UPDATE_PROJECT) }]);
  });

  it('preserves catalog order among roots', () => {
    const tree = buildPermissionTree(
      [def({ id: Permission.DELETE_PROJECT }), def({ id: Permission.READ_PROJECT })],
      label,
    );
    expect(tree.map(n => n.id)).toEqual([Permission.DELETE_PROJECT, Permission.READ_PROJECT]);
  });

  it('preserves catalog order among siblings sharing the same parent', () => {
    const tree = buildPermissionTree(
      [
        def({ id: Permission.READ_PROJECT }),
        def({ id: Permission.DELETE_PROJECT, dependsOn: Permission.READ_PROJECT }),
        def({ id: Permission.UPDATE_PROJECT, dependsOn: Permission.READ_PROJECT }),
      ],
      label,
    );

    expect(tree[0].children?.map(c => c.id)).toEqual([
      Permission.DELETE_PROJECT,
      Permission.UPDATE_PROJECT,
    ]);
  });

  it('an empty catalog produces an empty tree', () => {
    expect(buildPermissionTree([], label)).toEqual([]);
  });

  it('resolves each node\'s label through the given labeler', () => {
    const tree = buildPermissionTree([def({ id: Permission.READ_PROJECT })], () => 'Custom label');
    expect(tree[0].label).toBe('Custom label');
  });
});
