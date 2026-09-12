import { describe, it, expect, vi } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { I18nProvider } from '@lingui/react';
import { i18n } from '@lingui/core';
import { DependenciesProvider } from '@platform/di';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useCreateUser } from './use-create-user';
import { useDeleteUser } from './use-delete-user';
import { useUpdateUser } from './use-update-user';
import { useUsers } from './use-users';
import { useUser } from './use-user';
import type { UserRepository } from '@/modules/features/user/domain/repository/user.repository.ts';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';

const toastSuccess = vi.fn();
vi.mock('sonner', () => ({
  toast: { success: (...args: unknown[]) => toastSuccess(...args) },
}));

const user = (overrides: Partial<UserEntity> = {}): UserEntity => ({
  userId: 'user-1',
  username: 'ravenne',
  createdAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
});

const makeFakeRepository = (overrides: Partial<UserRepository> = {}) => {
  const getAll =
    overrides.getAll ??
    vi
      .fn()
      .mockResolvedValue(
        ScyllaResult.success({ items: [user()], pagination: { totalCount: 1, page: 1, pageSize: 10, totalPages: 1, hasNext: false, hasPrevious: false } }),
      );
  const getById = overrides.getById ?? vi.fn().mockResolvedValue(ScyllaResult.success(user()));
  const create = overrides.create ?? vi.fn().mockResolvedValue(ScyllaResult.success(user()));
  const update = overrides.update ?? vi.fn().mockResolvedValue(ScyllaResult.success(user()));
  const del = overrides.delete ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));

  const repository: UserRepository = { getAll, getById, create, update, delete: del };
  return { repository, getAll, getById, create, update, delete: del };
};

const wrapperFor = (repository: UserRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <I18nProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <DependenciesProvider registry={{ user: { userRepository: repository } }}>
          {children}
        </DependenciesProvider>
      </QueryClientProvider>
    </I18nProvider>
  );
  return { Wrapper, queryClient };
};

describe('useUsers', () => {
  it('lists the user directory', async () => {
    const { repository } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useUsers(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.users?.items).toHaveLength(1));
  });

  it('is enabled by default', () => {
    const { repository, getAll } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useUsers(), { wrapper: Wrapper });
    expect(getAll).toHaveBeenCalled();
  });

  it('does not fetch when the caller explicitly disables it (no LIST_USERS)', () => {
    const { repository, getAll } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useUsers({ enabled: false }), { wrapper: Wrapper });
    expect(getAll).not.toHaveBeenCalled();
  });

  it('surfaces a repository error', async () => {
    const error = new ScyllaError('boom', { cause: { code: 'PERMISSION_DENIED' } });
    const { repository } = makeFakeRepository({
      getAll: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useUsers(), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error).toBe(error);
  });
});

describe('useUser', () => {
  it('fetches a single user by id', async () => {
    const { repository, getById } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    const { result } = renderHook(() => useUser('user-1'), { wrapper: Wrapper });

    await waitFor(() => expect(result.current.user?.userId).toBe('user-1'));
    expect(getById).toHaveBeenCalledWith('user-1');
  });

  it('does not fetch when userId is undefined', () => {
    const { repository, getById } = makeFakeRepository();
    const { Wrapper } = wrapperFor(repository);
    renderHook(() => useUser(undefined), { wrapper: Wrapper });
    expect(getById).not.toHaveBeenCalled();
  });
});

describe('useCreateUser', () => {
  it('creates, toasts, and invalidates the users list', async () => {
    const { repository, create } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useCreateUser(), { wrapper: Wrapper });

    await result.current.mutateAsync({ username: 'new-user', password: 'hunter2' });

    expect(create).toHaveBeenCalledWith('new-user', 'hunter2');
    expect(toastSuccess).toHaveBeenCalledWith('User created');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['users'] });
  });
});

describe('useDeleteUser', () => {
  it('deletes, toasts, and invalidates the users list', async () => {
    const { repository, delete: del } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useDeleteUser(), { wrapper: Wrapper });

    await result.current.mutateAsync('user-1');

    expect(del).toHaveBeenCalledWith('user-1');
    expect(toastSuccess).toHaveBeenCalledWith('User deleted');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['users'] });
  });
});

describe('useUpdateUser', () => {
  it('updates and invalidates only that user\'s own detail key, NOT the users list', async () => {
    const { repository, update } = makeFakeRepository();
    const { Wrapper, queryClient } = wrapperFor(repository);
    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');
    const { result } = renderHook(() => useUpdateUser(), { wrapper: Wrapper });

    await result.current.mutateAsync({ userId: 'user-1', username: 'renamed' });

    expect(update).toHaveBeenCalledWith('user-1', 'renamed');
    expect(toastSuccess).toHaveBeenCalledWith('User information updated');
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: ['user', 'user-1'] });
    expect(invalidateSpy).not.toHaveBeenCalledWith({ queryKey: ['users'] });
  });
});
