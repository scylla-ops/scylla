import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { createMemoryRouter, RouterProvider } from 'react-router-dom';
import { usePermissionsStore, PermissionScope, Permission } from '@platform/authz';
import { RouteGuard } from './RouteGuard';
import type { RouteHandle } from './route-handle.struct.ts';

const grantEverything = () =>
  usePermissionsStore.setState({
    permissions: {
      scopes: [{ scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'fullControl' } }],
    },
  });

const grantOnly = (...permissions: Permission[]) =>
  usePermissionsStore.setState({
    permissions: {
      scopes: [
        { scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'restricted', permissions } },
      ],
    },
  });

/**
 * Mounts the guard the way the shell does: a pathless layout route carrying the
 * handle, with the guarded pages nested underneath.
 *
 * `useMatches` only works under a data router, which is also what the app uses
 * (see core/presentation/ui/router) — a plain `<MemoryRouter>` throws here.
 */
const renderGuarded = ({
  parentHandle,
  childHandle,
  path = '/secrets',
}: {
  parentHandle?: RouteHandle;
  childHandle?: RouteHandle;
  path?: string;
}) => {
  const router = createMemoryRouter(
    [
      {
        element: <RouteGuard />,
        handle: parentHandle,
        children: [
          { path: '/secrets', element: <span>secret list</span>, handle: childHandle },
          { path: '/secrets/new', element: <span>new secret</span>, handle: childHandle },
        ],
      },
    ],
    { initialEntries: [path] },
  );

  return render(
    <I18nProvider i18n={i18n}>
      <RouterProvider router={router} />
    </I18nProvider>,
  );
};

beforeEach(() => {
  usePermissionsStore.setState({ permissions: null });
});

describe('RouteGuard', () => {
  it('renders the route untouched when no match declares a permission', async () => {
    // Permissions are still null — an unguarded route must not wait on them.
    renderGuarded({});
    expect(await screen.findByText('secret list')).toBeInTheDocument();
  });

  it('lets the route through when the declared permission is held', async () => {
    grantEverything();
    renderGuarded({ parentHandle: { permission: Permission.LIST_SECRETS } });
    expect(await screen.findByText('secret list')).toBeInTheDocument();
  });

  it('replaces the route with a denial when the permission is missing', async () => {
    grantOnly(Permission.READ_PROJECT);
    renderGuarded({ parentHandle: { permission: Permission.LIST_SECRETS } });

    expect(await screen.findByText(/don't have the permission/i)).toBeInTheDocument();
    expect(screen.queryByText('secret list')).not.toBeInTheDocument();
  });

  it('shows neither the route nor a denial while permissions are unknown', () => {
    renderGuarded({ parentHandle: { permission: Permission.LIST_SECRETS } });

    expect(screen.queryByText('secret list')).not.toBeInTheDocument();
    expect(screen.queryByText(/don't have the permission/i)).not.toBeInTheDocument();
  });

  it('the deepest match wins: a child asking for more than its parent is checked against its own requirement', async () => {
    // Holds the parent's permission but not the child's.
    grantOnly(Permission.LIST_SECRETS);
    renderGuarded({
      parentHandle: { permission: Permission.LIST_SECRETS },
      childHandle: { permission: Permission.CREATE_SECRET },
      path: '/secrets/new',
    });

    expect(await screen.findByText(/don't have the permission/i)).toBeInTheDocument();
    expect(screen.queryByText('new secret')).not.toBeInTheDocument();
  });

  it('a child that declares nothing inherits its parent requirement rather than falling open', async () => {
    grantOnly(Permission.READ_PROJECT);
    renderGuarded({ parentHandle: { permission: Permission.LIST_SECRETS }, path: '/secrets/new' });

    expect(await screen.findByText(/don't have the permission/i)).toBeInTheDocument();
    expect(screen.queryByText('new secret')).not.toBeInTheDocument();
  });

  it('a handle carrying only a breadcrumb is not treated as a permission requirement', async () => {
    renderGuarded({ parentHandle: { breadcrumb: () => ({ label: { id: 'Secrets' } }) } });
    expect(await screen.findByText('secret list')).toBeInTheDocument();
  });
});
