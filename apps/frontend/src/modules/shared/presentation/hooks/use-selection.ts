import { useCallback, useMemo } from 'react';
import { useSelectionStore } from '@shared/presentation/stores/use-selection.store.ts';
import { useShallow } from 'zustand/react/shallow';

const EMPTY_ARRAY: string[] = [];

export const useSelection = (key: string) => {
  const selectedIds = useSelectionStore(useShallow(state => state.selectedIds[key] ?? EMPTY_ARRAY));
  const selectFn = useSelectionStore(state => state.select);
  const clearFn = useSelectionStore(state => state.clearSelection);

  const select = useCallback((id: string) => selectFn(key, id), [selectFn, key]);
  const clearSelection = useCallback(() => clearFn(key), [clearFn, key]);

  return useMemo(
    () => ({ selectedIds, select, clearSelection }),
    [selectedIds, select, clearSelection],
  );
};
