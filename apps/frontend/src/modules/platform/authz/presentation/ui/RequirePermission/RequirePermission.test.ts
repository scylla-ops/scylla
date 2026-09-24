import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import RequirePermissionFixture from './RequirePermission.fixture.svelte';

beforeEach(() => {
  permissionsStore.setState({ permissions: null });
});

describe('RequirePermission', () => {
  it('shows a spinner while the permissions are unknown, never the content or a denial', () => {
    render(RequirePermissionFixture, { permission: Permission.READ_PROJECT });

    expect(screen.queryByText('gated content')).not.toBeInTheDocument();
    expect(screen.queryByText(/don't have the permission/i)).not.toBeInTheDocument();
    expect(screen.getByRole('status')).toBeInTheDocument();
  });

  it('renders the content when the user holds the permission', () => {
    permissionsStore.setState({
      permissions: {
        scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
      },
    });
    render(RequirePermissionFixture, { permission: Permission.READ_PROJECT });

    expect(screen.getByText('gated content')).toBeInTheDocument();
  });

  it('renders the denial panel when the user lacks the permission', () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    render(RequirePermissionFixture, { permission: Permission.READ_PROJECT });

    expect(screen.queryByText('gated content')).not.toBeInTheDocument();
    expect(screen.getByText(/don't have the permission/i)).toBeInTheDocument();
  });

  it('shows a custom message in the denial panel', () => {
    permissionsStore.setState({ permissions: { scopes: [] } });
    render(RequirePermissionFixture, {
      permission: Permission.READ_PROJECT,
      message: 'Ask for manage-roles.',
    });

    expect(screen.getByText('Ask for manage-roles.')).toBeInTheDocument();
  });
});
