import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { PermissionButton } from './PermissionButton';
import { usePermissionsStore, PermissionScope, Permission } from '@platform/authz';

const renderWithI18n = (ui: React.ReactElement) =>
  render(<I18nProvider i18n={i18n}>{ui}</I18nProvider>);

// jsdom has no ResizeObserver, and Radix's Tooltip.Content measures itself
// with @radix-ui/react-use-size, which needs one to mount at all.
class ResizeObserverStub {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeEach(() => {
  usePermissionsStore.setState({ permissions: null });
  vi.stubGlobal('ResizeObserver', ResizeObserverStub);
});

describe('PermissionButton', () => {
  it('renders enabled when the user holds the permission', () => {
    usePermissionsStore.setState({
      permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
    });

    renderWithI18n(<PermissionButton permission={Permission.CREATE_PROJECT}>New project</PermissionButton>);

    expect(screen.getByRole('button', { name: 'New project' })).toBeEnabled();
  });

  it('disables quietly (no tooltip) while permissions are still unknown', () => {
    renderWithI18n(<PermissionButton permission={Permission.CREATE_PROJECT}>New project</PermissionButton>);

    const button = screen.getByRole('button', { name: 'New project' });
    expect(button).toBeDisabled();
    // No wrapper span/tooltip trigger - a real denial gets one, "still loading" does not.
    expect(button.closest('span.inline-flex')).toBeNull();
  });

  it('disables with an explanatory tooltip once denial is confirmed', async () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    const user = userEvent.setup();

    renderWithI18n(<PermissionButton permission={Permission.CREATE_PROJECT}>New project</PermissionButton>);

    const button = screen.getByRole('button', { name: 'New project' });
    expect(button).toBeDisabled();

    await user.hover(button);
    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent("don't have permission"));
  });

  it('shows a custom deniedReason instead of the default message', async () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });
    const user = userEvent.setup();

    renderWithI18n(
      <PermissionButton permission={Permission.CREATE_PROJECT} deniedReason='Ask an org admin.'>
        New project
      </PermissionButton>,
    );

    await user.hover(screen.getByRole('button', { name: 'New project' }));
    await waitFor(() => expect(screen.getByRole('tooltip')).toHaveTextContent('Ask an org admin.'));
  });

  it('an explicit disabled prop is respected when the user holds the permission', () => {
    usePermissionsStore.setState({
      permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
    });

    renderWithI18n(
      <PermissionButton permission={Permission.CREATE_PROJECT} disabled>
        New project
      </PermissionButton>,
    );

    expect(screen.getByRole('button', { name: 'New project' })).toBeDisabled();
  });
});
