// @vitest-environment node
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';
import { contextStore } from '@platform/context';
import { runMutationFn, runOnSuccess, runQueryFn } from '@/test/queries.ts';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import type { OrganizationEntity } from '../../domain/entities/organization.entity.ts';
import type { OrganizationRepository } from '../../domain/repository/organization.repository.ts';
import {
  invalidateOrganizationMembers,
  organizationMutations,
  organizationQueries,
  ORGANIZATIONS_QUERY_KEY,
  ORGANIZATION_MEMBERS_QUERY_KEY,
} from '../organization.queries.ts';

const toastSuccess = vi.fn();
vi.mock('svelte-sonner', () => ({ toast: { success: (...args: unknown[]) => toastSuccess(...args) } }));

const ORG_ID = 'org-1';

const org = (overrides: Partial<OrganizationEntity> = {}): OrganizationEntity => ({
  id: ORG_ID,
  name: 'Scylla Inc',
  description: 'a test org',
  ...overrides,
});

const withRepository = (overrides: Partial<OrganizationRepository> = {}) => {
  const repository = {
    getAll: vi.fn().mockResolvedValue(ScyllaResult.success([org()])),
    getMine: vi.fn().mockResolvedValue(ScyllaResult.success([org()])),
    listMembers: vi
      .fn()
      .mockResolvedValue(ScyllaResult.success([{ userId: 'user-1', username: 'ravenne' }])),
    create: vi.fn().mockResolvedValue(ScyllaResult.success(org())),
    update: vi.fn().mockResolvedValue(ScyllaResult.success(org())),
    delete: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
    ...overrides,
  };

  setDependencyRegistry({ organization: { organizationRepository: repository } });
  return repository;
};

let queryClient: QueryClient;

beforeEach(() => {
  toastSuccess.mockClear();
  contextStore.getState().reset();
  queryClient = new QueryClient();
  setQueryClient(queryClient);
});

afterEach(() => {
  setDependencyRegistry(null);
  setQueryClient(null);
});

describe('organizationQueries.mine', () => {
  it('asks for the organizations the user belongs to — getMine, never getAll', async () => {
    const repository = withRepository();

    const organizations = await runQueryFn(organizationQueries.mine());

    expect(repository.getMine).toHaveBeenCalled();
    expect(repository.getAll).not.toHaveBeenCalled();
    expect(organizations).toHaveLength(1);
  });

  it('surfaces a failure for the query to own', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    withRepository({ getMine: vi.fn().mockResolvedValue(ScyllaResult.error(error)) });

    await expect(runQueryFn(organizationQueries.mine())).rejects.toBe(error);
  });
});

describe('organizationQueries.members', () => {
  it('lists members for the given organization', async () => {
    const repository = withRepository();

    const members = await runQueryFn(organizationQueries.members(ORG_ID));

    expect(repository.listMembers).toHaveBeenCalledWith(ORG_ID);
    expect(members).toHaveLength(1);
  });

  it('is disabled without an organization id', () => {
    withRepository();
    expect(organizationQueries.members(null).enabled).toBe(false);
  });

  it('is disabled when the caller says so, even with a valid id', () => {
    withRepository();
    expect(organizationQueries.members(ORG_ID, { enabled: false }).enabled).toBe(false);
  });
});

describe('invalidateOrganizationMembers', () => {
  it("invalidates exactly this organization's members key", () => {
    withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    invalidateOrganizationMembers(ORG_ID);

    expect(invalidate).toHaveBeenCalledWith({
      queryKey: ORGANIZATION_MEMBERS_QUERY_KEY(ORG_ID),
      exact: true,
    });
  });

  it('is a no-op without an organization id', () => {
    withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    invalidateOrganizationMembers(null);

    expect(invalidate).not.toHaveBeenCalled();
  });
});

describe('organizationMutations.create', () => {
  it('creates the organization', async () => {
    const repository = withRepository();

    await runMutationFn(organizationMutations.create(), { name: 'New Org', description: 'd' });

    expect(repository.create).toHaveBeenCalledWith('New Org', 'd');
  });

  it('switches the active context to it, toasts and invalidates the list', () => {
    withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    runOnSuccess(organizationMutations.create(), org({ id: 'org-new', name: 'New Org' }), {
      name: 'New Org',
    });

    expect(contextStore.getState().organization).toEqual({ id: 'org-new', name: 'New Org' });
    expect(toastSuccess).toHaveBeenCalledWith('Organization created');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ORGANIZATIONS_QUERY_KEY() });
  });
});

describe('organizationMutations.update', () => {
  it('updates, toasts and invalidates the list', async () => {
    const repository = withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    await runMutationFn(organizationMutations.update(), {
      organizationId: ORG_ID,
      name: 'Renamed',
    });
    runOnSuccess(organizationMutations.update(), org(), { organizationId: ORG_ID });

    expect(repository.update).toHaveBeenCalledWith(ORG_ID, 'Renamed', undefined);
    expect(toastSuccess).toHaveBeenCalledWith('Organization updated');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ORGANIZATIONS_QUERY_KEY() });
  });
});

describe('organizationMutations.remove', () => {
  it('deletes, toasts and invalidates the list', async () => {
    const repository = withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    await runMutationFn(organizationMutations.remove(), ORG_ID);
    runOnSuccess(organizationMutations.remove(), undefined, ORG_ID);

    expect(repository.delete).toHaveBeenCalledWith(ORG_ID);
    expect(toastSuccess).toHaveBeenCalledWith('Organization deleted');
    expect(invalidate).toHaveBeenCalledWith({ queryKey: ORGANIZATIONS_QUERY_KEY() });
  });
});
