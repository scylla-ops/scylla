import { describe, it, expect, vi, beforeEach } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useFeatureSelection } from './use-feature-selection';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';

beforeEach(() => {
  useSelectionStore.setState({ selectedIds: {} });
});

describe('useFeatureSelection', () => {
  it('reports nothing selected and allSelected false when the list is empty', () => {
    const { result } = renderHook(() => useFeatureSelection('agents', []));
    expect(result.current.selectedCount).toBe(0);
    expect(result.current.allSelected).toBe(false);
  });

  it('isSelected reflects the underlying selection', () => {
    const { result } = renderHook(() => useFeatureSelection('agents', ['a', 'b']));
    act(() => result.current.select('a'));
    expect(result.current.isSelected('a')).toBe(true);
    expect(result.current.isSelected('b')).toBe(false);
  });

  it('allSelected becomes true once every id in the list is selected', () => {
    const { result } = renderHook(() => useFeatureSelection('agents', ['a', 'b']));
    act(() => result.current.select('a'));
    expect(result.current.allSelected).toBe(false);
    act(() => result.current.select('b'));
    expect(result.current.allSelected).toBe(true);
  });

  it('headerProps.onSelectAll selects every id in the given list', () => {
    const { result } = renderHook(() => useFeatureSelection('agents', ['a', 'b', 'c']));
    act(() => result.current.headerProps.onSelectAll());
    expect(result.current.selectedIds).toEqual(['a', 'b', 'c']);
  });

  it('headerProps.onClearSelection empties the selection', () => {
    const { result } = renderHook(() => useFeatureSelection('agents', ['a', 'b']));
    act(() => result.current.select('a'));
    act(() => result.current.headerProps.onClearSelection());
    expect(result.current.selectedCount).toBe(0);
  });

  it('omits onDeleteSelection from headerProps when no deleteItem is given', () => {
    const { result } = renderHook(() => useFeatureSelection('agents', ['a']));
    expect(result.current.headerProps.onDeleteSelection).toBeUndefined();
  });

  it('bulk-deletes every selected id and clears the selection afterwards', async () => {
    const deleteItem = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() =>
      useFeatureSelection('agents', ['a', 'b', 'c'], { deleteItem }),
    );
    act(() => {
      result.current.select('a');
      result.current.select('b');
    });

    await act(async () => {
      await result.current.headerProps.onDeleteSelection?.();
    });

    expect(deleteItem).toHaveBeenCalledTimes(2);
    expect(deleteItem).toHaveBeenCalledWith('a');
    expect(deleteItem).toHaveBeenCalledWith('b');
    expect(result.current.selectedCount).toBe(0);
  });

  it('still clears the selection even if one delete call rejects (Promise.allSettled semantics)', async () => {
    const deleteItem = vi.fn().mockRejectedValueOnce(new Error('nope')).mockResolvedValueOnce(undefined);
    const { result } = renderHook(() => useFeatureSelection('agents', ['a', 'b'], { deleteItem }));
    act(() => {
      result.current.select('a');
      result.current.select('b');
    });

    await act(async () => {
      await result.current.headerProps.onDeleteSelection?.();
    });

    expect(result.current.selectedCount).toBe(0);
  });
});
