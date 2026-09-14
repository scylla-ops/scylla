import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import { useResourceError } from './use-resource-error';
import { ScyllaError } from '@shared/utils/scylla-result.ts';

const navigateMock = vi.fn();
vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
}));

const toastErrorMock = vi.fn();
vi.mock('sonner', () => ({
  toast: { error: (...args: unknown[]) => toastErrorMock(...args) },
}));

beforeEach(() => {
  navigateMock.mockClear();
  toastErrorMock.mockClear();
});

describe('useResourceError', () => {
  it('does nothing for a non-ScyllaError', () => {
    const { result } = renderHook(() =>
      useResourceError({ error: new Error('boom'), redirectTo: '..', notFoundMessage: 'Gone' }),
    );
    expect(result.current.redirecting).toBe(false);
    expect(navigateMock).not.toHaveBeenCalled();
    expect(toastErrorMock).not.toHaveBeenCalled();
  });

  it('does nothing for a ScyllaError that is not NOT_FOUND', () => {
    const error = new ScyllaError('boom', { cause: { code: 'PERMISSION_DENIED' } });
    const { result } = renderHook(() =>
      useResourceError({ error, redirectTo: '..', notFoundMessage: 'Gone' }),
    );
    expect(result.current.redirecting).toBe(false);
    expect(navigateMock).not.toHaveBeenCalled();
  });

  it('toasts and redirects (replacing history) on NOT_FOUND', () => {
    const error = new ScyllaError('boom', { cause: { code: 'NOT_FOUND' } });
    const { result } = renderHook(() =>
      useResourceError({ error, redirectTo: '../list', notFoundMessage: 'Agent not found' }),
    );

    expect(result.current.redirecting).toBe(true);
    expect(toastErrorMock).toHaveBeenCalledWith('Agent not found');
    expect(navigateMock).toHaveBeenCalledWith('../list', { replace: true });
  });

  it('exposes the underlying ScyllaError for the caller to inspect further', () => {
    const error = new ScyllaError('boom', { cause: { code: 'NOT_FOUND' } });
    const { result } = renderHook(() =>
      useResourceError({ error, redirectTo: '..', notFoundMessage: 'Gone' }),
    );
    expect(result.current.scyllaError).toBe(error);
  });

  it('scyllaError is null for a plain error', () => {
    const { result } = renderHook(() =>
      useResourceError({ error: new Error('boom'), redirectTo: '..', notFoundMessage: 'Gone' }),
    );
    expect(result.current.scyllaError).toBeNull();
  });

  it('does not redirect at all when there is no error', () => {
    const { result } = renderHook(() =>
      useResourceError({ error: undefined, redirectTo: '..', notFoundMessage: 'Gone' }),
    );
    expect(result.current.redirecting).toBe(false);
    expect(navigateMock).not.toHaveBeenCalled();
  });
});
