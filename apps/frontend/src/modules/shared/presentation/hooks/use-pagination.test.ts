import { describe, it, expect } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { usePagination } from './use-pagination';

describe('usePagination', () => {
  it('defaults to page 1 and the default page size', () => {
    const { result } = renderHook(() => usePagination());
    expect(result.current.page).toBe(1);
    expect(result.current.pageSize).toBe(10);
    expect(result.current.paginationParams).toEqual({ page: 1, pageSize: 10 });
  });

  it('honors initial page / page size', () => {
    const { result } = renderHook(() => usePagination({ initialPage: 3, initialPageSize: 25 }));
    expect(result.current.page).toBe(3);
    expect(result.current.pageSize).toBe(25);
  });

  it('setPage updates the page', () => {
    const { result } = renderHook(() => usePagination());
    act(() => result.current.setPage(4));
    expect(result.current.page).toBe(4);
  });

  it('setPage clamps below 1 up to 1', () => {
    const { result } = renderHook(() => usePagination({ initialPage: 5 }));
    act(() => result.current.setPage(0));
    expect(result.current.page).toBe(1);
    act(() => result.current.setPage(-3));
    expect(result.current.page).toBe(1);
  });

  it('setPageSize clamps to [1, 100] and resets to page 1', () => {
    const { result } = renderHook(() => usePagination({ initialPage: 5 }));

    act(() => result.current.setPageSize(500));
    expect(result.current.pageSize).toBe(100);
    expect(result.current.page).toBe(1);

    act(() => result.current.setPage(3));
    act(() => result.current.setPageSize(0));
    expect(result.current.pageSize).toBe(1);
    expect(result.current.page).toBe(1);
  });

  it('paginationInfo is undefined until the server has reported pagination data', () => {
    const { result } = renderHook(() => usePagination());
    expect(result.current.paginationInfo).toBeUndefined();
  });

  it('updatePaginationInfo derives hasNext/hasPrevious from the current page', () => {
    const { result } = renderHook(() => usePagination({ initialPage: 2 }));
    act(() =>
      result.current.updatePaginationInfo({
        totalCount: 42,
        page: 2,
        pageSize: 10,
        totalPages: 5,
        hasNext: true,
        hasPrevious: true,
      }),
    );
    expect(result.current.paginationInfo).toEqual({
      totalCount: 42,
      totalPages: 5,
      page: 2,
      pageSize: 10,
      hasNext: true,
      hasPrevious: true,
    });
  });

  it('paginationInfo.hasPrevious is false on page 1', () => {
    const { result } = renderHook(() => usePagination());
    act(() =>
      result.current.updatePaginationInfo({
        totalCount: 3,
        page: 1,
        pageSize: 10,
        totalPages: 1,
        hasNext: false,
        hasPrevious: false,
      }),
    );
    expect(result.current.paginationInfo?.hasPrevious).toBe(false);
    expect(result.current.paginationInfo?.hasNext).toBe(false);
  });

  it('snaps back to the last page when the current page no longer exists (e.g. after deleting the last item on it)', () => {
    const { result } = renderHook(() => usePagination({ initialPage: 5 }));
    act(() =>
      result.current.updatePaginationInfo({
        totalCount: 30,
        page: 5,
        pageSize: 10,
        totalPages: 5,
        hasNext: false,
        hasPrevious: true,
      }),
    );
    expect(result.current.page).toBe(5);

    // Server now reports fewer pages than we're sitting on.
    act(() =>
      result.current.updatePaginationInfo({
        totalCount: 20,
        page: 5,
        pageSize: 10,
        totalPages: 2,
        hasNext: false,
        hasPrevious: true,
      }),
    );
    expect(result.current.page).toBe(2);
  });

  it('does nothing to the page when totalPages is 0 (empty result set, not an overshoot)', () => {
    const { result } = renderHook(() => usePagination({ initialPage: 3 }));
    act(() =>
      result.current.updatePaginationInfo({
        totalCount: 0,
        page: 3,
        pageSize: 10,
        totalPages: 0,
        hasNext: false,
        hasPrevious: false,
      }),
    );
    expect(result.current.page).toBe(3);
  });
});
