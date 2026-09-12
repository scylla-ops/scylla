import { describe, it, expect, vi } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { DependenciesProvider } from '@platform/di';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useSecrets, useCreateSecret, useDeleteSecret } from './use-secrets';
import type { SecretRepository } from '@/modules/features/secret/domain/repository/secret.repository.ts';
import type { SecretEntity } from '@/modules/features/secret/domain/entities/secret.entity.ts';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

const PROJECT_ID = 'project-1';

const secret = (overrides: Partial<SecretEntity> = {}): SecretEntity => ({
  id: 'secret-1',
  projectId: PROJECT_ID,
  name: 'API_KEY',
  description: 'a test secret',
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const makeFakeRepository = (overrides: Partial<SecretRepository> = {}) => {
  const listByProjectId = vi.fn().mockResolvedValue(ScyllaResult.success([secret()]));
  const create = vi.fn().mockResolvedValue(ScyllaResult.success(secret()));
  const deleteById = vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  const repository: SecretRepository = { listByProjectId, create, deleteById, ...overrides };
  return { repository, listByProjectId, create, deleteById };
};

const wrapperFor = (repository: SecretRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <I18nProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <DependenciesProvider registry={{ secret: { secretRepository: repository } }}>
          {children}
        </DependenciesProvider>
      </QueryClientProvider>
    </I18nProvider>
  );
  return { Wrapper, queryClient };
};

describe('useSecrets', () => {
  it('lists a project\'s secrets', async () => {
    const { repository, listByProjectId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useSecrets(PROJECT_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.secrets).toHaveLength(1));
    expect(listByProjectId).toHaveBeenCalledWith(PROJECT_ID);
  });

  it('does not fetch when projectId is empty', () => {
    const { repository, listByProjectId } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useSecrets(''), { wrapper: Wrapper });

    expect(listByProjectId).not.toHaveBeenCalled();
  });

  it('surfaces a repository error via isError/error', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'PERMISSION_DENIED' } });
    const { repository } = makeFakeRepository({
      listByProjectId: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useSecrets(PROJECT_ID), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBe(error);
  });
});

describe('useCreateSecret', () => {
  it('creates a secret scoped to the project, toasts, and invalidates the list', async () => {
    const { repository, create } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateSecret(PROJECT_ID), { wrapper: Wrapper });

    await result.current.mutateAsync({ name: 'TOKEN', value: 'secret-value', description: 'd' });

    expect(create).toHaveBeenCalledWith({
      projectId: PROJECT_ID,
      name: 'TOKEN',
      value: 'secret-value',
      description: 'd',
    });
    expect(toastSuccess).toHaveBeenCalledWith('Secret created');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['secrets', PROJECT_ID] });
  });

  it('rejects instead of toasting when the repository call fails', async () => {
    const error = new ScyllaError('name taken', { cause: { code: 'ALREADY_EXISTS' } });
    const { repository } = makeFakeRepository({
      create: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useCreateSecret(PROJECT_ID), { wrapper: Wrapper });

    await expect(
      result.current.mutateAsync({ name: 'DUP', value: 'v', description: '' }),
    ).rejects.toBe(error);
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});

describe('useDeleteSecret', () => {
  it('deletes by id and invalidates the project\'s secret list (no toast)', async () => {
    const { repository, deleteById } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteSecret(PROJECT_ID), { wrapper: Wrapper });

    await result.current.mutateAsync('secret-1');

    expect(deleteById).toHaveBeenCalledWith('secret-1');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['secrets', PROJECT_ID] });
    expect(toastSuccess).not.toHaveBeenCalled();
  });
});
