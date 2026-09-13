import { describe, it, expect, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import userEvent from '@testing-library/user-event';
import { RoleDialogPermissions } from './RoleDialogPermissions';
import { PermissionScope } from '@platform/authz';

describe('RoleDialogPermissions', () => {
  it('counts the implicit "member of" permission for an ORGANIZATION role with nothing ticked', () => {
    renderWithI18n(
      <RoleDialogPermissions
        scope={PermissionScope.ORGANIZATION}
        permissions={[]}
        preservedCount={0}
        isPending={false}
        onPermissionsChange={vi.fn()}
      />,
    );
    // ORGANIZATION always confers READ_ORGANIZATION - one selected, none ticked.
    expect(screen.getByText('1 selected')).toBeInTheDocument();
    expect(screen.getByText('Always')).toBeInTheDocument();
  });

  it('a PROJECT role has no always-granted permissions, so the section and its explanation are both absent', () => {
    renderWithI18n(
      <RoleDialogPermissions
        scope={PermissionScope.PROJECT}
        permissions={[]}
        preservedCount={0}
        isPending={false}
        onPermissionsChange={vi.fn()}
      />,
    );
    expect(screen.getByText('0 selected')).toBeInTheDocument();
    expect(screen.queryByText('Always')).not.toBeInTheDocument();
    expect(
      screen.queryByText(/Holding a role in an organization is what belonging to it means/),
    ).not.toBeInTheDocument();
  });

  it('shows a note about preserved (unmanaged) permissions only when there are some', () => {
    const { rerender } = renderWithI18n(
      <RoleDialogPermissions
        scope={PermissionScope.PROJECT}
        permissions={[]}
        preservedCount={0}
        isPending={false}
        onPermissionsChange={vi.fn()}
      />,
    );
    expect(screen.queryByText(/not managed here/)).not.toBeInTheDocument();

    rerender(
      <RoleDialogPermissions
        scope={PermissionScope.PROJECT}
        permissions={[]}
        preservedCount={2}
        isPending={false}
        onPermissionsChange={vi.fn()}
      />,

    );
    expect(screen.getByText(/not managed here/)).toBeInTheDocument();
  });

  it('ticking a permission in the tree forwards the new selection', async () => {
    const onPermissionsChange = vi.fn();
    const user = userEvent.setup();
    renderWithI18n(
      <RoleDialogPermissions
        scope={PermissionScope.PROJECT}
        permissions={[]}
        preservedCount={0}
        isPending={false}
        onPermissionsChange={onPermissionsChange}
      />,
    );

    const [firstCheckbox] = screen.getAllByRole('checkbox');
    await user.click(firstCheckbox);
    expect(onPermissionsChange).toHaveBeenCalled();
  });

  it('disables every checkbox in the tree while pending', () => {
    renderWithI18n(
      <RoleDialogPermissions
        scope={PermissionScope.PROJECT}
        permissions={[]}
        preservedCount={0}
        isPending
        onPermissionsChange={vi.fn()}
      />,
    );
    for (const checkbox of screen.getAllByRole('checkbox')) {
      expect(checkbox).toBeDisabled();
    }
  });
});
