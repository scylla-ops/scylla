import { useCallback, useMemo } from 'react';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';
import { useShallow } from 'zustand/react/shallow';

const EMPTY_ARRAY: string[] = [];

export const useSelection = (key: string) => {
  const selectedIds = useSelectionStore(useShallow(state => state.selectedIds[key] ?? EMPTY_ARRAY));
  const selectFn = useSelectionStore(state => state.select);
  const selectAllFn = useSelectionStore(state => state.selectAll);
  const clearFn = useSelectionStore(state => state.clearSelection);

  const select = useCallback((id: string) => selectFn(key, id), [selectFn, key]);
  const selectAll = useCallback((ids: string[]) => selectAllFn(key, ids), [selectAllFn, key]);
  const clearSelection = useCallback(() => clearFn(key), [clearFn, key]);

  return useMemo(
    () => ({ selectedIds, select, selectAll, clearSelection }),
    [selectedIds, select, selectAll, clearSelection],
  );
};
