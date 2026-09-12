import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { DependenciesProvider } from '@platform/di';
import { useContextStore } from '@platform/context';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useApps, useApp, useAppSecrets } from './use-apps';
import type { AppsRepository } from '@/modules/features/apps/domain/repository/apps.repository.ts';
import type { AppEntity, AppSecretEntity } from '@/modules/features/apps/domain/entities/app.entity.ts';

const ORG_ID = 'org-1';
const APP_ID = 'app-1';

const app = (overrides: Partial<AppEntity> = {}): AppEntity => ({
  id: APP_ID,
  organizationId: ORG_ID,
  name: 'ci-runner',
  isActive: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const appSecret = (overrides: Partial<AppSecretEntity> = {}): AppSecretEntity => ({
  id: 'secret-1',
  appId: APP_ID,
  label: 'default',
  enabled: true,
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const makeFakeRepository = (overrides: Partial<AppsRepository> = {}) => {
  const listApps = vi.fn().mockResolvedValue(ScyllaResult.success([app()]));
  const getApp = vi.fn().mockResolvedValue(ScyllaResult.success(app()));
  const createApp = vi.fn().mockResolvedValue(ScyllaResult.success({ app: app(), secret: 's' }));
  const deleteApp = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const setAppActive = vi
    .fn()
    .mockImplementation((id: string, active: boolean) =>
      Promise.resolve(ScyllaResult.success(app({ id, isActive: active }))),
    );
  const listAppSecrets = vi.fn().mockResolvedValue(ScyllaResult.success([appSecret()]));
  const createAppSecret = vi
    .fn()
    .mockResolvedValue(ScyllaResult.success({ credential: appSecret(), secret: 's3cr3t' }));
  const revokeAppSecret = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const setAppSecretEnabled = vi
    .fn()
    .mockImplementation((id: string, enabled: boolean) =>
      Promise.resolve(ScyllaResult.success(appSecret({ id, enabled }))),
    );

  const repository: AppsRepository = {
    listApps,
    getApp,
    createApp,
    deleteApp,
    setAppActive,
    listAppSecrets,
    createAppSecret,
    revokeAppSecret,
    setAppSecretEnabled,
    ...overrides,
  };
  return {
    repository,
    listApps,
    getApp,
    createApp,
    deleteApp,
    setAppActive,
    listAppSecrets,
    createAppSecret,
    revokeAppSecret,
    setAppSecretEnabled,
  };
};

const wrapperFor = (repository: AppsRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <DependenciesProvider registry={{ apps: { appsRepository: repository } }}>
        {children}
      </DependenciesProvider>
    </QueryClientProvider>
  );
  return { Wrapper, queryClient };
};

beforeEach(() => {
  useContextStore.setState({ organization: { id: ORG_ID, name: 'Org' } });
});

describe('useApps', () => {
  it('lists the current organization\'s apps', async () => {
    const { repository, listApps } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useApps(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.apps).toHaveLength(1));
    expect(listApps).toHaveBeenCalledWith(ORG_ID);
  });

  it('surfaces a repository error', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    const { repository } = makeFakeRepository({
      listApps: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useApps(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBe(error);
  });

  it('createApp invalidates the org-scoped apps list', async () => {
    const { repository, createApp } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useApps(), { wrapper: Wrapper });

    await result.current.createApp.mutateAsync('new-app');

    expect(createApp).toHaveBeenCalledWith(ORG_ID, 'new-app');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['apps', ORG_ID] });
  });

  it('deleteApp invalidates the org-scoped apps list', async () => {
    const { repository, deleteApp } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useApps(), { wrapper: Wrapper });

    await result.current.deleteApp.mutateAsync(APP_ID);

    expect(deleteApp).toHaveBeenCalledWith(APP_ID);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['apps', ORG_ID] });
  });

  it('setAppActive invalidates BOTH the list and that app\'s detail query', async () => {
    const { repository, setAppActive } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useApps(), { wrapper: Wrapper });

    await result.current.setAppActive.mutateAsync({ appId: APP_ID, active: false });

    expect(setAppActive).toHaveBeenCalledWith(APP_ID, false);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['apps', ORG_ID] });
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['apps', 'detail', APP_ID] });
  });
});

describe('useApp', () => {
  it('fetches a single app by id', async () => {
    const { repository, getApp } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useApp(APP_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.data?.id).toBe(APP_ID));
    expect(getApp).toHaveBeenCalledWith(APP_ID);
  });

  it('does not fetch when appId is empty', () => {
    const { repository, getApp } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useApp(''), { wrapper: Wrapper });
    expect(getApp).not.toHaveBeenCalled();
  });
});

describe('useAppSecrets', () => {
  it('lists an app\'s secrets', async () => {
    const { repository, listAppSecrets } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAppSecrets(APP_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.secrets).toHaveLength(1));
    expect(listAppSecrets).toHaveBeenCalledWith(APP_ID);
  });

  it('createSecret invalidates the app\'s secrets list', async () => {
    const { repository, createAppSecret } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useAppSecrets(APP_ID), { wrapper: Wrapper });

    const created = await result.current.createSecret.mutateAsync('deploy-key');

    expect(createAppSecret).toHaveBeenCalledWith(APP_ID, 'deploy-key');
    expect(created.secret).toBe('s3cr3t');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['app-secrets', APP_ID] });
  });

  it('revokeSecret invalidates the app\'s secrets list', async () => {
    const { repository, revokeAppSecret } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useAppSecrets(APP_ID), { wrapper: Wrapper });

    await result.current.revokeSecret.mutateAsync('secret-1');

    expect(revokeAppSecret).toHaveBeenCalledWith('secret-1');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['app-secrets', APP_ID] });
  });

  it('setSecretEnabled invalidates the app\'s secrets list', async () => {
    const { repository, setAppSecretEnabled } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useAppSecrets(APP_ID), { wrapper: Wrapper });

    await result.current.setSecretEnabled.mutateAsync({ secretId: 'secret-1', enabled: false });

    expect(setAppSecretEnabled).toHaveBeenCalledWith('secret-1', false);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['app-secrets', APP_ID] });
  });

  it('a failed mutation rejects instead of silently succeeding', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    const { repository } = makeFakeRepository({
      createAppSecret: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useAppSecrets(APP_ID), { wrapper: Wrapper });

    await expect(result.current.createSecret.mutateAsync('x')).rejects.toBe(error);
  });
});
