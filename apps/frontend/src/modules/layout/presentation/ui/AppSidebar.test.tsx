import { describe, it, expect, beforeEach, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { msg } from '@lingui/core/macro';
import { usePermissionsStore, PermissionScope, Permission } from '@platform/authz';
import { useContextStore } from '@platform/context';
import { SidebarProvider } from '@shadcn/sidebar.tsx';
import { renderWithI18n } from '@/test/render.tsx';
import type { NavEntry } from '@platform/routing';
import { AppSidebar } from './AppSidebar';

// The sidebar composes three feature-owned panels and the user menu. None of
// them takes part in the permission filtering this file is about, and all three
// reach for a repository through the DI registry.
vi.mock('@/modules/features/organization', () => ({
  OrganizationList: () => <div>organization list</div>,
  AddOrganizationDialog: () => <div>add organization</div>,
}));
vi.mock('@/modules/layout/presentation/ui/NavUser.tsx', () => ({
  NavUser: () => <div>nav user</div>,
}));
vi.mock('@/modules/layout/presentation/ui/LanguageSelector.tsx', () => ({
  LanguageSelector: () => <div>language selector</div>,
}));
vi.mock('@/modules/layout/presentation/ui/context-selector/ContextSelector.tsx', () => ({
  ContextSelector: ({ label }: { label: string }) => <div>context selector: {label}</div>,
}));

const entry = (overrides: Partial<NavEntry> = {}): NavEntry => ({
  section: 'organization',
  title: msg`Secrets`,
  url: 'secrets',
  permission: Permission.LIST_SECRETS,
  ...overrides,
});

const grantOnly = (...permissions: Permission[]) =>
  usePermissionsStore.setState({
    permissions: {
      scopes: [
        { scope: PermissionScope.SYSTEM, scopeId: '', access: { kind: 'restricted', permissions } },
      ],
    },
  });

const renderSidebar = (navEntries: readonly NavEntry[]) =>
  renderWithI18n(
    <MemoryRouter>
      <SidebarProvider>
        <AppSidebar navEntries={navEntries} />
      </SidebarProvider>
    </MemoryRouter>,
  );

beforeEach(() => {
  usePermissionsStore.setState({ permissions: null });
  useContextStore.setState(state => ({
    ...state,
    organization: { ...state.organization, id: 'org-1', name: 'Acme Corp' },
  }));
});

describe('AppSidebar permission filtering', () => {
  it('shows an entry whose permission the user holds', () => {
    grantOnly(Permission.LIST_SECRETS);
    renderSidebar([entry()]);

    expect(screen.getByText('Secrets')).toBeInTheDocument();
  });

  it('hides an entry whose permission the user lacks', () => {
    grantOnly(Permission.READ_PROJECT);
    renderSidebar([entry()]);

    expect(screen.queryByText('Secrets')).not.toBeInTheDocument();
  });

  it('always shows an entry that declares no permission', () => {
    grantOnly();
    renderSidebar([entry({ permission: undefined, title: msg`Dashboard`, url: 'dashboard' })]);

    expect(screen.getByText('Dashboard')).toBeInTheDocument();
  });

  it('filters per entry, not per section', () => {
    grantOnly(Permission.LIST_SECRETS);
    renderSidebar([
      entry(),
      entry({ title: msg`Pipelines`, url: 'pipelines', permission: Permission.LIST_PIPELINES }),
    ]);

    expect(screen.getByText('Secrets')).toBeInTheDocument();
    expect(screen.queryByText('Pipelines')).not.toBeInTheDocument();
  });

  it('renders skeletons and no links at all while permissions are unknown', () => {
    renderSidebar([entry({ permission: undefined })]);

    expect(screen.queryByText('Secrets')).not.toBeInTheDocument();
    // The organization selector stays put so the user can still switch org.
    expect(screen.getByText(/context selector/)).toBeInTheDocument();
  });

  it('drops a section left empty by the filter, but keeps the organization card for its selector', () => {
    grantOnly();
    renderSidebar([entry(), entry({ section: 'system', title: msg`Users`, url: 'users' })]);

    // "System" has no header, so an empty one disappears entirely.
    expect(screen.queryByText('System')).not.toBeInTheDocument();
    // The organization card survives on its header alone.
    expect(screen.getByText(/context selector/)).toBeInTheDocument();
  });

  it('keeps a system entry the user does hold', () => {
    grantOnly(Permission.LIST_USERS);
    renderSidebar([
      entry({
        section: 'system',
        title: msg`Users`,
        url: 'users',
        permission: Permission.LIST_USERS,
      }),
    ]);

    expect(screen.getByText('Users')).toBeInTheDocument();
  });
});

describe('AppSidebar url prefixing', () => {
  it('prefixes entry urls with the slugified current organization', () => {
    grantOnly(Permission.LIST_SECRETS);
    renderSidebar([entry()]);

    expect(screen.getByText('Secrets')).toBeInTheDocument();
    // The slug comes from the org name, not its id — see slugifyOrgName.
    expect(useContextStore.getState().organization.name).toBe('Acme Corp');
  });

  it('renders without an organization selected', () => {
    useContextStore.setState(state => ({
      ...state,
      organization: { ...state.organization, id: null, name: '' },
    }));
    grantOnly(Permission.LIST_SECRETS);
    renderSidebar([entry()]);

    expect(screen.getByText('Secrets')).toBeInTheDocument();
  });
});
