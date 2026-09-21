import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { Permission, PermissionScope } from '@platform/authz';
import { render } from '@/test/render.svelte.ts';
import RoleDialogPermissions from './RoleDialogPermissions.svelte';

/**
 * The permission picker — the densest piece of UI logic in the project.
 *
 * It needs no DI and no query client: the tree is built from the local
 * catalog, and the component's whole job is to emit a permission list.
 */
interface PickerProps {
  scope: PermissionScope;
  permissions: Permission[];
  preservedCount: number;
  conferredCount: number;
  isPending: boolean;
}

const mount = (props: Partial<PickerProps> = {}) => {
  const onPermissionsChange = vi.fn();
  render(RoleDialogPermissions, {
    scope: PermissionScope.ORGANIZATION,
    permissions: [],
    preservedCount: 0,
    conferredCount: 1,
    isPending: false,
    onPermissionsChange,
    ...props,
  });
  return { onPermissionsChange };
};

/** The last list the picker emitted. */
const emitted = (onPermissionsChange: ReturnType<typeof vi.fn>): Permission[] =>
  onPermissionsChange.mock.calls.at(-1)?.[0] as Permission[];

describe('RoleDialogPermissions', () => {
  it('shows what the scope confers by construction, ticked and locked', () => {
    mount();

    const membership = screen.getByRole('checkbox', { name: 'Member of the organization' });
    expect(membership).toBeChecked();
    expect(membership).toBeDisabled();
    expect(screen.getByText('Always')).toBeInTheDocument();
  });

  it('reports the honest count — riders included, not the boxes ticked', () => {
    mount({ conferredCount: 4 });

    expect(screen.getByText('4 selected')).toBeInTheDocument();
  });

  it('says how many permissions it is carrying that it cannot show', () => {
    mount({ preservedCount: 2 });

    expect(
      screen.getByText(
        'This role also holds 2 permission(s) not managed here. They are kept unchanged.',
      ),
    ).toBeInTheDocument();
  });

  it('stays quiet about preserved permissions when there are none', () => {
    mount();

    expect(screen.queryByText(/not managed here/)).not.toBeInTheDocument();
  });

  it('emits the permission when a root box is ticked', async () => {
    const { onPermissionsChange } = mount({ scope: PermissionScope.PROJECT });

    await userEvent.click(screen.getByRole('checkbox', { name: 'Member of the project' }));

    expect(emitted(onPermissionsChange)).toContain(Permission.READ_PROJECT);
  });

  it('leaves a child disabled until its parent is checked — it confers nothing alone', () => {
    mount({ scope: PermissionScope.PROJECT });

    expect(screen.getByRole('checkbox', { name: 'View project secrets' })).toBeDisabled();
  });

  it('enables a child once its parent is checked', () => {
    mount({ scope: PermissionScope.PROJECT, permissions: [Permission.READ_PROJECT] });

    expect(screen.getByRole('checkbox', { name: 'View project secrets' })).toBeEnabled();
  });

  it('ignores a seeded child whose parent chain is broken', () => {
    // LIST_SECRETS without READ_PROJECT confers nothing, so it must not read
    // as checked — showing it ticked would promise access the backend refuses.
    mount({ scope: PermissionScope.PROJECT, permissions: [Permission.LIST_SECRETS] });

    expect(screen.getByRole('checkbox', { name: 'View project secrets' })).not.toBeChecked();
  });

  it('clears the whole subtree when a parent is unchecked', async () => {
    const { onPermissionsChange } = mount({
      scope: PermissionScope.PROJECT,
      permissions: [Permission.READ_PROJECT, Permission.LIST_SECRETS, Permission.CREATE_SECRET],
    });

    await userEvent.click(screen.getByRole('checkbox', { name: 'Member of the project' }));

    const next = emitted(onPermissionsChange);
    expect(next).not.toContain(Permission.READ_PROJECT);
    expect(next).not.toContain(Permission.LIST_SECRETS);
    expect(next).not.toContain(Permission.CREATE_SECRET);
  });

  it('never offers a permission the scope cannot confer', () => {
    mount();

    // A system capability — meaningless on an organization role.
    expect(screen.queryByRole('checkbox', { name: 'View users' })).not.toBeInTheDocument();
  });

  it('offers the system capabilities once the scope is system', () => {
    mount({ scope: PermissionScope.SYSTEM });

    expect(screen.getByRole('checkbox', { name: 'View users' })).toBeInTheDocument();
  });

  it('hides the permission a node stands in for, rather than showing it twice', () => {
    mount({ scope: PermissionScope.SYSTEM });

    // MANAGE_SYSTEM_GRANTS rides on MANAGE_ROLES at this scope.
    expect(
      screen.queryByRole('checkbox', { name: 'Grant and revoke roles anywhere' }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole('checkbox', { name: 'Manage roles, and grant them anywhere' }),
    ).toBeInTheDocument();
  });

  it('gives its expand control a name, which the React original never had', async () => {
    mount();

    const collapse = screen.getAllByRole('button', { name: /sub-permissions$/ })[0];
    expect(collapse).toHaveAttribute('aria-expanded', 'true');

    await userEvent.click(collapse);
    expect(collapse).toHaveAttribute('aria-expanded', 'false');
  });

  it('hides a subtree when its node is collapsed', async () => {
    mount({ scope: PermissionScope.PROJECT, permissions: [Permission.READ_PROJECT] });

    await userEvent.click(
      screen.getByRole('button', { name: 'Hide Member of the project sub-permissions' }),
    );

    expect(
      screen.queryByRole('checkbox', { name: 'View project secrets' }),
    ).not.toBeInTheDocument();
  });

  it('disables every box while a write is in flight, without altering the selection', () => {
    mount({
      scope: PermissionScope.PROJECT,
      permissions: [Permission.READ_PROJECT],
      isPending: true,
    });

    const member = screen.getByRole('checkbox', { name: 'Member of the project' });
    expect(member).toBeChecked();
    expect(member).toBeDisabled();
  });
});
