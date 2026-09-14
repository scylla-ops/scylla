import { describe, it, expect, beforeEach } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useSelection } from './use-selection';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';

beforeEach(() => {
  useSelectionStore.setState({ selectedIds: {} });
});

describe('useSelection', () => {
  it('starts with no selection for a fresh key', () => {
    const { result } = renderHook(() => useSelection('jobs'));
    expect(result.current.selectedIds).toEqual([]);
  });

  it('select() adds an id', () => {
    const { result } = renderHook(() => useSelection('jobs'));
    act(() => result.current.select('job-1'));
    expect(result.current.selectedIds).toEqual(['job-1']);
  });

  it('select() on an already-selected id toggles it off', () => {
    const { result } = renderHook(() => useSelection('jobs'));
    act(() => result.current.select('job-1'));
    act(() => result.current.select('job-1'));
    expect(result.current.selectedIds).toEqual([]);
  });

  it('selectAll() replaces the selection wholesale', () => {
    const { result } = renderHook(() => useSelection('jobs'));
    act(() => result.current.select('job-1'));
    act(() => result.current.selectAll(['job-2', 'job-3']));
    expect(result.current.selectedIds).toEqual(['job-2', 'job-3']);
  });

  it('clearSelection() empties it', () => {
    const { result } = renderHook(() => useSelection('jobs'));
    act(() => result.current.selectAll(['job-1', 'job-2']));
    act(() => result.current.clearSelection());
    expect(result.current.selectedIds).toEqual([]);
  });

  it('two different keys are independent', () => {
    const jobs = renderHook(() => useSelection('jobs'));
    const users = renderHook(() => useSelection('users'));

    act(() => jobs.result.current.select('job-1'));

    expect(jobs.result.current.selectedIds).toEqual(['job-1']);
    expect(users.result.current.selectedIds).toEqual([]);
  });
});
