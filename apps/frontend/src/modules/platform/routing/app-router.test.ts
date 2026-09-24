import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { msg } from '@lingui/core/macro';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { currentPathname, navigateTo, setAppNavigator } from '@platform/context';
import { createAppRouter } from './app-router.ts';
import type { AppRouterConfig } from './app-route.struct.ts';
import { routePathname, routeTrail } from './route-state.ts';
import type { PageLoader } from './scylla-module.struct.ts';
import TestFallback from './TestFallback.fixture.svelte';
import TestOtherPage from './TestOtherPage.fixture.svelte';
import TestPage from './TestPage.fixture.svelte';
import TestShell from './TestShell.fixture.svelte';
import TestWrapper from './TestWrapper.fixture.svelte';
import { Router } from 'sv-router';

const page: PageLoader = () => Promise.resolve({ default: TestPage });
const otherPage: PageLoader = () => Promise.resolve({ default: TestOtherPage });

const config: AppRouterConfig = {
  publicRoutes: [{ path: '/login', lazy: otherPage }],
  shell: TestShell,
  fallback: TestFallback,
  routes: [
    {
      path: ':slug',
      wrapper: TestWrapper,
      children: [
        { index: true, redirect: 'secrets' },
        {
          path: 'secrets',
          handle: { permission: Permission.LIST_SECRETS, breadcrumb: () => ({ label: msg`Secrets` }) },
          children: [
            { index: true, lazy: page },
            { path: 'new', handle: { permission: Permission.CREATE_SECRET }, lazy: page },
            { path: ':secretId', lazy: page },
          ],
        },
        { path: 'open', handle: { breadcrumb: () => ({ label: msg`Open` }) }, lazy: otherPage },
      ],
    },
  ],
};

const grantOnly = (...permissions: Permission[]) =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        { scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'restricted', permissions } },
      ],
    },
  });

const renderAt = async (pathname: string) => {
  const navigator = createAppRouter(config);
  setAppNavigator(navigator);
  navigator.navigate(pathname, { replace: true });
  await expect.poll(() => routePathname()).toBe(pathname);
  return render(Router);
};

beforeEach(() => {
  permissionsStore.setState({ permissions: null });
});

afterEach(() => {
  setAppNavigator(null);
});

describe('the app router', () => {
  it('renders the page of the URL inside the shell and its wrappers', async () => {
    await renderAt('/acme/open');

    expect(await screen.findByTestId('other-page')).toBeInTheDocument();
    expect(screen.getByTestId('shell')).toContainElement(screen.getByTestId('wrapper'));
    expect(screen.getByTestId('wrapper')).toHaveAttribute('data-slug', 'acme');
  });

  it('gives the route parameters to the page as props', async () => {
    grantOnly(Permission.LIST_SECRETS);
    await renderAt('/acme/secrets/secret-1');

    expect(await screen.findByTestId('page')).toHaveTextContent('slug=acme secretId=secret-1');
  });

  it('renders a public route without the shell', async () => {
    await renderAt('/login');

    expect(await screen.findByTestId('other-page')).toBeInTheDocument();
    expect(screen.queryByTestId('shell')).not.toBeInTheDocument();
  });

  it('renders the fallback, without the shell, for a URL that no route matches', async () => {
    await renderAt('/acme/nothing/here');

    expect(await screen.findByTestId('fallback')).toBeInTheDocument();
    expect(screen.queryByTestId('shell')).not.toBeInTheDocument();
  });

  it('follows a redirect relative to the route', async () => {
    grantOnly(Permission.LIST_SECRETS);
    await renderAt('/acme');

    expect(await screen.findByTestId('page')).toHaveTextContent('slug=acme');
    expect(currentPathname()).toBe('/acme/secrets');
  });

  it('shows the page of the new URL after a navigation', async () => {
    await renderAt('/acme/open');
    await screen.findByTestId('other-page');

    grantOnly(Permission.LIST_SECRETS);
    navigateTo('/acme/secrets/secret-2');

    expect(await screen.findByTestId('page')).toHaveTextContent('secretId=secret-2');
  });

  it('resolves a relative navigation from the current page', async () => {
    grantOnly(Permission.LIST_SECRETS);
    await renderAt('/acme/secrets/secret-1');
    await screen.findByTestId('page');

    navigateTo('..');

    await expect.poll(() => currentPathname()).toBe('/acme/secrets');
  });

  it('keeps the query string apart from the pathname', async () => {
    await renderAt('/acme/open');
    await screen.findByTestId('other-page');

    navigateTo('/acme/open?nodes=build,test', { replace: true });

    await expect.poll(() => window.location.search).toBe('?nodes=build,test');
    expect(currentPathname()).toBe('/acme/open');
  });
});

describe('the route guard', () => {
  it('renders the page when no route on the URL declares a permission', async () => {
    await renderAt('/acme/open');

    expect(await screen.findByTestId('other-page')).toBeInTheDocument();
  });

  it('shows neither the page nor a denial while the permissions are unknown', async () => {
    await renderAt('/acme/secrets');
    await screen.findByRole('status');

    expect(screen.queryByTestId('page')).not.toBeInTheDocument();
    expect(screen.queryByText(/don't have the permission/i)).not.toBeInTheDocument();
  });

  it('replaces the page with a denial when the permission is missing', async () => {
    grantOnly(Permission.READ_PROJECT);
    await renderAt('/acme/secrets');

    expect(await screen.findByText(/don't have the permission/i)).toBeInTheDocument();
    expect(screen.queryByTestId('page')).not.toBeInTheDocument();
  });

  it('checks a child that asks for more than its parent against its own permission', async () => {
    grantOnly(Permission.LIST_SECRETS);
    await renderAt('/acme/secrets/new');

    expect(await screen.findByText(/don't have the permission/i)).toBeInTheDocument();
    expect(screen.queryByTestId('page')).not.toBeInTheDocument();
  });

  it('applies the permission of the parent to a child that declares none', async () => {
    grantOnly(Permission.READ_PROJECT);
    await renderAt('/acme/secrets/secret-1');

    expect(await screen.findByText(/don't have the permission/i)).toBeInTheDocument();
  });

  it('does not treat a handle with only a breadcrumb as a permission', async () => {
    await renderAt('/acme/open');

    expect(await screen.findByTestId('other-page')).toBeInTheDocument();
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});

describe('the route trail', () => {
  it('lists the handles on the URL with the pathname of their route', async () => {
    grantOnly(Permission.LIST_SECRETS, Permission.CREATE_SECRET);
    await renderAt('/acme/secrets/new');
    await screen.findByTestId('page');

    expect(routeTrail().map(crumb => crumb.pathname)).toEqual(['/acme/secrets', '/acme/secrets/new']);
  });
});
