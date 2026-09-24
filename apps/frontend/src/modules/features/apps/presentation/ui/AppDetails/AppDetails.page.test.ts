import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { type Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { installTestNavigator } from '@/test/navigator.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppsRepository } from '../../../domain/repository/apps.repository.ts';
import AppDetailsPage from './AppDetails.page.svelte';

const app = (overrides: Record<string, unknown> = {}) => ({
  id: 'app-1',
  organizationId: 'org-1',
  name: 'deploy-bot',
  isActive: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-02T00:00:00.000Z',
  ...overrides,
});

const secret = (overrides: Record<string, unknown> = {}) => ({
  id: 'secret-1',
  appId: 'app-1',
  label: 'ci',
  enabled: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

let getApp: ReturnType<typeof vi.fn>;
let deleteApp: ReturnType<typeof vi.fn>;
let setAppActive: ReturnType<typeof vi.fn>;
let listAppSecrets: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;
let nav: ReturnType<typeof installTestNavigator>;

const grant = (permissions: Permission[] | 'all') =>
  permissionsStore.setState({
    permissions: {
      scopes: [
        {
          scope: PermissionScope.SYSTEM,
          scopeId: '',
          access:
            permissions === 'all' ? { kind: 'fullControl' } : { kind: 'restricted', permissions },
        },
      ],
    },
  });

beforeEach(() => {
  getApp = vi.fn().mockResolvedValue(ScyllaResult.success(app()));
  deleteApp = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  setAppActive = vi.fn().mockResolvedValue(ScyllaResult.success(app({ isActive: false })));
  listAppSecrets = vi.fn().mockResolvedValue(ScyllaResult.success([secret()]));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    apps: {
      appsRepository: {
        getApp,
        deleteApp,
        setAppActive,
        listAppSecrets,
        listApps: vi.fn(),
        createApp: vi.fn(),
        createAppSecret: vi.fn(),
        revokeAppSecret: vi.fn(),
        setAppSecretEnabled: vi.fn(),
      } as unknown as AppsRepository,
    },
  });

  contextStore.setState({
    organization: { id: 'org-1', name: 'Acme' },
    project: { id: null, name: null },
  });
  nav = installTestNavigator({ pathname: '/acme/apps/app-1' });
  grant('all');
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  nav.restore();
  permissionsStore.setState({ permissions: null });
});

describe('AppDetailsPage', () => {
  it('reads the app named by the route parameter', async () => {
    render(AppDetailsPage, { appId: 'app-1' });

    expect(await screen.findByRole('heading', { name: 'deploy-bot' })).toBeInTheDocument();
    expect(getApp).toHaveBeenCalledWith('app-1');
  });

  it('says plainly that an app is a credential, not a process to monitor', async () => {
    render(AppDetailsPage, { appId: 'app-1' });

    expect(await screen.findByText(/Apps are credentials, not processes/)).toBeInTheDocument();
  });

  it('lists the app’s secrets', async () => {
    render(AppDetailsPage, { appId: 'app-1' });

    expect(await screen.findByText('ci')).toBeInTheDocument();
    expect(listAppSecrets).toHaveBeenCalledWith('app-1');
  });

  it('toggles the app active state from the switch', async () => {
    render(AppDetailsPage, { appId: 'app-1' });
    await screen.findByRole('heading', { name: 'deploy-bot' });

    await userEvent.click(screen.getByRole('switch', { name: 'Toggle app active' }));

    await vi.waitFor(() =>
      expect(setAppActive).toHaveBeenCalledWith('app-1', false),
    );
  });

  it('reports a failed load rather than a blank page', async () => {
    getApp.mockResolvedValue(
      ScyllaResult.error(new ScyllaError('boom', { cause: { code: 'INTERNAL' } })),
    );
    render(AppDetailsPage, { appId: 'app-1' });

    expect(await screen.findByText('Error loading app')).toBeInTheDocument();
  });

  it('leaves for the list when the app no longer exists', async () => {
    getApp.mockResolvedValue(
      ScyllaResult.error(new ScyllaError('gone', { cause: { code: 'NOT_FOUND' } })),
    );
    render(AppDetailsPage, { appId: 'app-1' });

    await vi.waitFor(() => expect(nav.navigate).toHaveBeenCalledWith('..', expect.anything()));
  });

  it('confirms before deleting, then returns to the list', async () => {
    render(AppDetailsPage, { appId: 'app-1' });
    await screen.findByRole('heading', { name: 'deploy-bot' });

    await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
    expect(await screen.findByText('Delete app?')).toBeInTheDocument();
    expect(deleteApp).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await vi.waitFor(() => expect(deleteApp).toHaveBeenCalledWith('app-1'));
  });

  it('asks for nothing at all without an app id', () => {
    render(AppDetailsPage, {});

    expect(getApp).not.toHaveBeenCalled();
  });
});
