import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ReactNode } from 'react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { renderHook, waitFor } from '@testing-library/react';
import { DependenciesProvider } from '@platform/di';
import { ScyllaResult, ScyllaError } from '@shared/utils/scylla-result.ts';
import { useLogin } from './use-login';
import type { LoginRepository } from '@/modules/features/login/domain/repository/login.repository.ts';

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
}));

beforeEach(() => {
  navigateMock.mockClear();
});

const makeFakeRepository = (overrides: Partial<LoginRepository> = {}) => {
  const login = overrides.login ?? vi.fn().mockResolvedValue(ScyllaResult.success(undefined));
  const repository: LoginRepository = { login };
  return { repository, login };
};

const wrapperFor = (repository: LoginRepository) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <DependenciesProvider registry={{ login: { loginRepository: repository } }}>
        {children}
      </DependenciesProvider>
    </QueryClientProvider>
  );
  return Wrapper;
};

describe('useLogin', () => {
  it('logs in with the given credentials and redirects to the root, replacing history', async () => {
    const { repository, login } = makeFakeRepository();
    const { result } = renderHook(() => useLogin(), { wrapper: wrapperFor(repository) });

    await result.current.mutateAsync({ login: 'ravenne', password: 'hunter2' });

    expect(login).toHaveBeenCalledWith('ravenne', 'hunter2');
    expect(navigateMock).toHaveBeenCalledWith('/', { replace: true });
  });

  it('does not navigate when the login call fails', async () => {
    const error = new ScyllaError('bad credentials', { cause: { code: 'INVALID_CREDENTIALS' } });
    const { repository } = makeFakeRepository({
      login: vi.fn().mockResolvedValue(ScyllaResult.error(error)),
    });
    const { result } = renderHook(() => useLogin(), { wrapper: wrapperFor(repository) });

    await expect(
      result.current.mutateAsync({ login: 'ravenne', password: 'wrong' }),
    ).rejects.toBe(error);

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(navigateMock).not.toHaveBeenCalled();
  });
});
