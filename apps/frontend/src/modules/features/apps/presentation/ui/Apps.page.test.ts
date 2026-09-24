import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { type Permission, PermissionScope, permissionsStore } from '@platform/authz';
import { contextStore } from '@platform/context';
import { focusSettled, render, withQueryClient, withRegistry } from '@/test/render.svelte.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppsRepository } from '../../domain/repository/apps.repository.ts';
import AppsPage from './Apps.page.svelte';

const app = (overrides: Record<string, unknown> = {}) => ({
  id: 'app-1',
  organizationId: 'org-1',
  name: 'deploy-bot',
  isActive: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

let listApps: ReturnType<typeof vi.fn>;
let createApp: ReturnType<typeof vi.fn>;
let deleteApp: ReturnType<typeof vi.fn>;
let cache: ReturnType<typeof withQueryClient>;
let restoreRegistry: () => void;

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
  listApps = vi.fn().mockResolvedValue(ScyllaResult.success([app()]));
  createApp = vi
    .fn()
    .mockResolvedValue(ScyllaResult.success({ app: app({ id: 'app-new' }), secret: 'sk-once' }));
  deleteApp = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  cache = withQueryClient();
  restoreRegistry = withRegistry({
    apps: {
      appsRepository: {
        listApps,
        createApp,
        deleteApp,
        getApp: vi.fn(),
        setAppActive: vi.fn(),
        listAppSecrets: vi.fn(),
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
  grant('all');
});

afterEach(() => {
  cache.restore();
  restoreRegistry();
  permissionsStore.setState({ permissions: null });
});

describe('AppsPage', () => {
  it("lists the organization's apps", async () => {
    render(AppsPage);

    expect(await screen.findByText('deploy-bot')).toBeInTheDocument();
    expect(listApps).toHaveBeenCalledWith('org-1');
  });

  it('explains the empty state instead of showing a bare list', async () => {
    listApps.mockResolvedValue(ScyllaResult.success([]));
    render(AppsPage);

    expect(await screen.findByText('No apps yet')).toBeInTheDocument();
  });

  it('offers to create the first app when the caller may', async () => {
    listApps.mockResolvedValue(ScyllaResult.success([]));
    render(AppsPage);

    expect(await screen.findByRole('button', { name: 'Create your first app' })).toBeInTheDocument();
  });

  it('reports a failed load rather than an empty list', async () => {
    listApps.mockResolvedValue(
      ScyllaResult.error(new ScyllaError('boom', { cause: { code: 'INTERNAL' } })),
    );
    render(AppsPage);

    expect(await screen.findByText('Error loading apps')).toBeInTheDocument();
  });

  it('creates an app and reveals its one-time secret', async () => {
    render(AppsPage);
    await screen.findByText('deploy-bot');

    await userEvent.click(screen.getAllByRole('button', { name: 'New App' })[0]);
    await focusSettled();
    await userEvent.type(await screen.findByLabelText(/name/i), 'ci-bot');
    await userEvent.click(screen.getByRole('button', { name: 'Create & reveal secret →' }));

    await vi.waitFor(() => expect(createApp).toHaveBeenCalledWith('org-1', 'ci-bot'));
    expect(await screen.findByText('sk-once')).toBeInTheDocument();
  });

  it('refuses to create an app with a blank name', async () => {
    render(AppsPage);
    await screen.findByText('deploy-bot');

    await userEvent.click(screen.getAllByRole('button', { name: 'New App' })[0]);
    await focusSettled();
    await userEvent.type(await screen.findByLabelText(/name/i), '   ');
    await userEvent.click(screen.getByRole('button', { name: 'Create & reveal secret →' }));

    expect(createApp).not.toHaveBeenCalled();
  });

  it('asks for confirmation before deleting, and deletes only on continue', async () => {
    render(AppsPage);
    await screen.findByText('deploy-bot');

    await userEvent.click(screen.getByRole('button', { name: 'Delete app' }));
    expect(await screen.findByText('Delete app?')).toBeInTheDocument();
    expect(deleteApp).not.toHaveBeenCalled();

    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await vi.waitFor(() => expect(deleteApp).toHaveBeenCalledWith('app-1'));
  });
});
