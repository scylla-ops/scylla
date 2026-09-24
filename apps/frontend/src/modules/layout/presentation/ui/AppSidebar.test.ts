import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { msg } from '@lingui/core/macro';
import { Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import type { NavEntry } from '@platform/routing';
import { installTestNavigator } from '@/test/navigator.ts';
import AppSidebarFixture from './AppSidebar.fixture.svelte';

vi.mock('./context-selector/OrganizationSelector.svelte', async () => ({
  default: (await import('./Stub.fixture.svelte')).default,
}));
vi.mock('./NavUser.svelte', async () => ({
  default: (await import('./Stub.fixture.svelte')).default,
}));
vi.mock('./LanguageSelector.svelte', async () => ({
  default: (await import('./Stub.fixture.svelte')).default,
}));

const entry = (overrides: Partial<NavEntry> = {}): NavEntry => ({
  section: 'organization',
  title: msg`Secrets`,
  url: 'secrets',
  permission: Permission.LIST_SECRETS,
  ...overrides,
});

const grantOnly = (...permissions: Permission[]) =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        { scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'restricted', permissions } },
      ],
    },
  });

let navigator: ReturnType<typeof installTestNavigator>;

beforeEach(() => {
  navigator = installTestNavigator({ pathname: '/acme-corp/dashboard' });
  localStorage.clear();
  permissionsStore.setState({ permissions: null });
  contextStore.setState({ organization: { id: 'org-1', name: 'Acme Corp' } });
});

afterEach(() => navigator.restore());

describe('AppSidebar', () => {
  it('shows the entries the user may open', () => {
    grantOnly(Permission.LIST_SECRETS);
    render(AppSidebarFixture, { navEntries: [entry()] });

    expect(screen.getByRole('button', { name: /Secrets/ })).toBeInTheDocument();
  });

  it('hides an entry whose permission the user lacks', () => {
    grantOnly(Permission.READ_PROJECT);
    render(AppSidebarFixture, { navEntries: [entry()] });

    expect(screen.queryByRole('button', { name: /Secrets/ })).not.toBeInTheDocument();
  });

  it('shows skeletons and no entry while the permissions are unknown, but keeps the selector', () => {
    render(AppSidebarFixture, { navEntries: [entry({ permission: undefined })] });

    expect(screen.queryByRole('button', { name: /Secrets/ })).not.toBeInTheDocument();
    expect(screen.getAllByText('stub').length).toBeGreaterThan(0);
  });

  it('opens the page of an entry under the current organization', async () => {
    grantOnly(Permission.LIST_SECRETS);
    render(AppSidebarFixture, { navEntries: [entry()] });

    await userEvent.click(screen.getByRole('button', { name: /Secrets/ }));

    expect(navigator.navigate).toHaveBeenCalledWith('/acme-corp/secrets', undefined);
  });

  it('marks the release highlight of an entry as seen when the user opens it', async () => {
    grantOnly();
    render(AppSidebarFixture, {
      navEntries: [entry({ permission: undefined, title: msg`Dashboard`, url: 'dashboard' })],
    });
    expect(screen.getByText('New')).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: /Dashboard/ }));

    expect(screen.queryByText('New')).not.toBeInTheDocument();
  });
});
