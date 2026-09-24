// @vitest-environment node
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';
import { runMutationFn, runOnSuccess, runQueryFn } from '@/test/queries.ts';
import { ScyllaError, ScyllaResult } from '@shared/utils/scylla-result.ts';
import type { AppsRepository } from '../../domain/repository/apps.repository.ts';
import {
  APPS_QUERY_KEY,
  APP_QUERY_KEY,
  APP_SECRETS_QUERY_KEY,
  appMutations,
  appQueries,
} from '../apps.queries.ts';

const app = {
  id: 'app-1',
  organizationId: 'org-1',
  name: 'ci-runner',
  isActive: true,
  createdAt: '',
  updatedAt: '',
};

let repository: AppsRepository;
let queryClient: QueryClient;
let invalidate: ReturnType<typeof vi.fn>;

beforeEach(() => {
  repository = {
    listApps: vi.fn().mockResolvedValue(ScyllaResult.success([app])),
    getApp: vi.fn().mockResolvedValue(ScyllaResult.success(app)),
    createApp: vi.fn().mockResolvedValue(ScyllaResult.success({ app, secret: 'sk-once' })),
    deleteApp: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
    setAppActive: vi.fn().mockResolvedValue(ScyllaResult.success(app)),
    listAppSecrets: vi.fn().mockResolvedValue(ScyllaResult.success([])),
    createAppSecret: vi.fn().mockResolvedValue(ScyllaResult.success({ credential: {}, secret: 's' })),
    revokeAppSecret: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
    setAppSecretEnabled: vi.fn().mockResolvedValue(ScyllaResult.success({})),
  };

  setDependencyRegistry({ apps: { appsRepository: repository } });

  queryClient = new QueryClient();
  invalidate = vi.fn();
  queryClient.invalidateQueries = invalidate as unknown as QueryClient['invalidateQueries'];
  setQueryClient(queryClient);
});

afterEach(() => {
  setDependencyRegistry(null);
  setQueryClient(null);
});

describe('appQueries', () => {
  it('lists the apps of one organization', async () => {
    const options = appQueries.byOrganization('org-1');

    expect(options.queryKey).toEqual(APPS_QUERY_KEY('org-1'));
    await expect(runQueryFn(options)).resolves.toEqual([app]);
    expect(repository.listApps).toHaveBeenCalledWith('org-1');
  });

  it('stays disabled without an organization, rather than fetching for an empty id', () => {
    expect(appQueries.byOrganization('').enabled).toBe(false);
    expect(appQueries.byId('').enabled).toBe(false);
    expect(appQueries.secretsOf('').enabled).toBe(false);
  });

  it('keeps an app and its secrets in separate cache entries', () => {
    // Different keys: a secret mutation invalidates the secrets, not the detail.
    expect(APP_QUERY_KEY('app-1')).not.toEqual(APP_SECRETS_QUERY_KEY('app-1'));
  });

  it('surfaces a repository failure so TanStack Query owns the error', async () => {
    vi.mocked(repository.getApp).mockResolvedValue(ScyllaResult.error(new ScyllaError('gone')));

    await expect(runQueryFn(appQueries.byId('app-1'))).rejects.toThrow('gone');
  });
});

describe('appMutations', () => {
  it('creates an app and invalidates the list it now belongs to', async () => {
    const options = appMutations.create('org-1');

    const created = await runMutationFn(options, 'ci-runner');
    expect(repository.createApp).toHaveBeenCalledWith('org-1', 'ci-runner');
    expect(created.secret).toBe('sk-once');

    runOnSuccess(options, created, 'ci-runner');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: APPS_QUERY_KEY('org-1') });
  });

  it('invalidates both the list and the detail after a toggle, since both show the flag', async () => {
    const options = appMutations.setActive('org-1');
    const variables = { appId: 'app-1', active: false };

    await runMutationFn(options, variables);
    expect(repository.setAppActive).toHaveBeenCalledWith('app-1', false);

    runOnSuccess(options, app, variables);
    expect(invalidate).toHaveBeenCalledWith({ queryKey: APPS_QUERY_KEY('org-1') });
    expect(invalidate).toHaveBeenCalledWith({ queryKey: APP_QUERY_KEY('app-1') });
  });

  it('revoking a secret refreshes that app secrets list and nothing else', async () => {
    const options = appMutations.revokeSecret('app-1');

    await runMutationFn(options, 'secret-9');
    expect(repository.revokeAppSecret).toHaveBeenCalledWith('secret-9');

    runOnSuccess(options, undefined, 'secret-9');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: APP_SECRETS_QUERY_KEY('app-1') });
    expect(invalidate).not.toHaveBeenCalledWith({ queryKey: APPS_QUERY_KEY('org-1') });
  });

  it('keeps disabling a secret distinct from revoking it', async () => {
    await runMutationFn(appMutations.setSecretEnabled('app-1'), {
      secretId: 'secret-9',
      enabled: false,
    });

    // Disabling is reversible, revoking is not: two distinct calls.
    expect(repository.setAppSecretEnabled).toHaveBeenCalledWith('secret-9', false);
    expect(repository.revokeAppSecret).not.toHaveBeenCalled();
  });
});
