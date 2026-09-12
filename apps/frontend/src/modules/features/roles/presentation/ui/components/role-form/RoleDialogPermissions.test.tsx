import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { RoleDialogPermissions } from './RoleDialogPermissions';
import { PermissionScope } from '@platform/authz';

class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

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
      <I18nProvider i18n={i18n}>
        <RoleDialogPermissions
          scope={PermissionScope.PROJECT}
          permissions={[]}
          preservedCount={2}
          isPending={false}
          onPermissionsChange={vi.fn()}
        />
      </I18nProvider>,
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
