import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useContextStore } from '@platform/context';
import { usePermissionSync } from './use-permission-sync';

const refreshMock = vi.fn().mockResolvedValue(undefined);
vi.mock('./use-refresh-my-permissions', () => ({
  useRefreshMyPermissions: () => refreshMock,
}));

beforeEach(() => {
  refreshMock.mockClear();
  useContextStore.getState().reset();
  localStorage.clear();
});

describe('usePermissionSync', () => {
  it('refreshes once on mount (the initial login sync)', () => {
    renderHook(() => usePermissionSync());
    expect(refreshMock).toHaveBeenCalledTimes(1);
  });

  it('does not refresh again on a re-render where nothing relevant changed (e.g. StrictMode replay)', () => {
    const { rerender } = renderHook(() => usePermissionSync());
    expect(refreshMock).toHaveBeenCalledTimes(1);

    rerender();
    expect(refreshMock).toHaveBeenCalledTimes(1);
  });

  it('refreshes again when the active organization changes', () => {
    const { rerender } = renderHook(() => usePermissionSync());
    expect(refreshMock).toHaveBeenCalledTimes(1);

    act(() => useContextStore.getState().setOrganization('org-1', 'Org'));
    rerender();

    expect(refreshMock).toHaveBeenCalledTimes(2);
  });

  it('refreshes again when the active project changes', () => {
    act(() => useContextStore.getState().setOrganization('org-1', 'Org'));
    const { rerender } = renderHook(() => usePermissionSync());
    expect(refreshMock).toHaveBeenCalledTimes(1);

    act(() => useContextStore.getState().setProject('project-1', 'Project'));
    rerender();

    expect(refreshMock).toHaveBeenCalledTimes(2);
  });

  it('a userId change alone (org/project unchanged) is picked up the next time the effect actually runs, not on an unrelated re-render', () => {
    // localStorage isn't a React dependency here - only read imperatively
    // inside the effect - so changing it alone must NOT force a re-sync on a
    // bare re-render with the same org/project. That guard is exactly what
    // stops StrictMode's double-invoke from re-fetching. Real user switches
    // pair with an org/project reset (e.g. on login/logout), which does
    // change a dependency and is covered by the two tests above.
    const { rerender } = renderHook(() => usePermissionSync());
    expect(refreshMock).toHaveBeenCalledTimes(1);

    localStorage.setItem('userId', 'user-2');
    rerender();

    expect(refreshMock).toHaveBeenCalledTimes(1);
  });
});
