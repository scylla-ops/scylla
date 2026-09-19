// @vitest-environment node
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { QueryClient } from '@tanstack/query-core';
import { setDependencyRegistry } from '@platform/di';
import { setQueryClient } from '@platform/query';
import { runMutationFn, runOnSuccess, runQueryFn } from '@/test/queries.ts';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import type { SecretEntity } from '../domain/entities/secret.entity.ts';
import type { SecretRepository } from '../domain/repository/secret.repository.ts';
import { SECRETS_QUERY_KEY, secretMutations, secretQueries } from './secret.queries.ts';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({ toast: { success: (...args: unknown[]) => toastSuccess(...args) } }));

const secret = (overrides: Partial<SecretEntity> = {}): SecretEntity => ({
  id: 'secret-1',
  projectId: 'project-1',
  name: 'DATABASE_URL',
  description: 'prod db',
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const withRepository = (overrides: Partial<SecretRepository> = {}) => {
  const repository = {
    listByProjectId: vi.fn().mockResolvedValue(ScyllaResult.success([secret()])),
    create: vi.fn().mockResolvedValue(ScyllaResult.success(secret())),
    deleteById: vi.fn().mockResolvedValue(ScyllaResult.success(undefined)),
    ...overrides,
  } as unknown as SecretRepository;
  setDependencyRegistry({ secret: { secretRepository: repository } });
  return repository;
};

let queryClient: QueryClient;

beforeEach(() => {
  toastSuccess.mockClear();
  queryClient = new QueryClient();
  setQueryClient(queryClient);
});

afterEach(() => {
  setDependencyRegistry(null);
  setQueryClient(null);
});

describe('secretQueries.byProject', () => {
  it('lists the project’s secrets through the repository', async () => {
    const repository = withRepository();

    const secrets = await runQueryFn(secretQueries.byProject('project-1'));

    expect(repository.listByProjectId).toHaveBeenCalledWith('project-1');
    expect(secrets).toEqual([secret()]);
  });

  it('stays disabled without a project id, rather than asking for every secret', () => {
    withRepository();
    expect(secretQueries.byProject('').enabled).toBe(false);
  });

  it('surfaces a repository failure for the query to own', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'INTERNAL' } });
    withRepository({ listByProjectId: vi.fn().mockResolvedValue(ScyllaResult.error(error)) });

    await expect(runQueryFn(secretQueries.byProject('project-1'))).rejects.toBe(error);
  });
});

describe('secretMutations.create', () => {
  it('scopes the new secret to the project the factory was given', async () => {
    const repository = withRepository();

    await runMutationFn(secretMutations.create('project-42'), {
      name: 'DATABASE_URL',
      value: 'postgres://…',
      description: 'prod db',
    });

    expect(repository.create).toHaveBeenCalledWith({
      projectId: 'project-42',
      name: 'DATABASE_URL',
      value: 'postgres://…',
      description: 'prod db',
    });
  });

  it('toasts and invalidates that project’s list on success', () => {
    withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    runOnSuccess(secretMutations.create('project-1'), secret(), {
      name: 'a',
      value: 'b',
      description: '',
    });

    expect(toastSuccess).toHaveBeenCalled();
    expect(invalidate).toHaveBeenCalledWith({ queryKey: SECRETS_QUERY_KEY('project-1') });
  });
});

describe('secretMutations.remove', () => {
  it('deletes by id', async () => {
    const repository = withRepository();

    await runMutationFn(secretMutations.remove('project-1'), 'secret-9');

    expect(repository.deleteById).toHaveBeenCalledWith('secret-9');
  });

  it('invalidates the list but does not toast — the two call sites word it differently', () => {
    withRepository();
    const invalidate = vi.spyOn(queryClient, 'invalidateQueries');

    runOnSuccess(secretMutations.remove('project-1'), undefined, 'secret-9');

    expect(invalidate).toHaveBeenCalledWith({ queryKey: SECRETS_QUERY_KEY('project-1') });
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});
