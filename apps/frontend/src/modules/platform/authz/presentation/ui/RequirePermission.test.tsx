import { describe, it, expect, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { renderWithI18n } from '@/test/render.tsx';
import { RequirePermission } from './RequirePermission';
import { usePermissionsStore, PermissionScope, Permission } from '@platform/authz';

beforeEach(() => {
  usePermissionsStore.setState({ permissions: null });
});

describe('RequirePermission', () => {
  it('shows a quiet spinner while permissions are still unknown - never the gated content, never a denial', () => {
    renderWithI18n(
      <RequirePermission permission={Permission.READ_PROJECT}>
        <span>gated content</span>
      </RequirePermission>,
    );

    expect(screen.queryByText('gated content')).not.toBeInTheDocument();
    expect(screen.queryByText(/don't have the permission/i)).not.toBeInTheDocument();
    expect(screen.getByRole('status')).toBeInTheDocument();
  });

  it('renders children once the user is confirmed to hold the permission', () => {
    usePermissionsStore.setState({
      permissions: { scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }] },
    });

    renderWithI18n(
      <RequirePermission permission={Permission.READ_PROJECT}>
        <span>gated content</span>
      </RequirePermission>,
    );

    expect(screen.getByText('gated content')).toBeInTheDocument();
  });

  it('renders the PermissionDenied panel once permissions are known and the user lacks it', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });

    renderWithI18n(
      <RequirePermission permission={Permission.READ_PROJECT}>
        <span>gated content</span>
      </RequirePermission>,
    );

    expect(screen.queryByText('gated content')).not.toBeInTheDocument();
    expect(screen.getByText(/don't have the permission/i)).toBeInTheDocument();
  });

  it('forwards a custom message into the denial panel', () => {
    usePermissionsStore.setState({ permissions: { scopes: [] } });

    renderWithI18n(
      <RequirePermission permission={Permission.READ_PROJECT} message='Ask for manage-roles.'>
        <span>gated content</span>
      </RequirePermission>,
    );

    expect(screen.getByText('Ask for manage-roles.')).toBeInTheDocument();
  });
});
