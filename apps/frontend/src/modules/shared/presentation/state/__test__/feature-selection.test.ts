// @vitest-environment node
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { createFeatureSelection } from '../feature-selection.svelte.ts';

beforeEach(() => selectionStore.setState({ selectedIds: {} }));

const ids = ['a', 'b', 'c'];

describe('createFeatureSelection', () => {
  it('reports nothing selected and allSelected false when the list is empty', () => {
    const feature = createFeatureSelection('jobs', () => []);

    expect(feature.selectedCount).toBe(0);
    expect(feature.allSelected).toBe(false);
  });

  it('reflects the underlying selection through isSelected', () => {
    const feature = createFeatureSelection('jobs', () => ids);

    feature.select('b');

    expect(feature.isSelected('b')).toBe(true);
    expect(feature.isSelected('a')).toBe(false);
  });

  it('turns allSelected true once every id in the list is selected', () => {
    const feature = createFeatureSelection('jobs', () => ids);

    feature.selectAll();

    expect(feature.allSelected).toBe(true);
    expect(feature.selectedCount).toBe(3);
  });

  it('re-reads allIds, so a list that arrives later is the one select-all uses', () => {
    let rows: string[] = [];
    const feature = createFeatureSelection('jobs', () => rows);

    // The React hook re-ran on every render and got this for free; here it is
    // the getter that does it. Taking the array once would freeze select-all on
    // the empty first render.
    rows = ids;
    feature.selectAll();

    expect(feature.selectedIds).toEqual(ids);
  });

  it('empties the selection through headerProps.onClearSelection', () => {
    const feature = createFeatureSelection('jobs', () => ids);
    feature.selectAll();

    feature.headerProps.onClearSelection();

    expect(feature.selectedCount).toBe(0);
  });

  it('omits onDeleteSelection from headerProps when no deleteItem is given', () => {
    const feature = createFeatureSelection('jobs', () => ids);

    expect(feature.headerProps.onDeleteSelection).toBeUndefined();
  });

  it('bulk-deletes every selected id and clears the selection afterwards', async () => {
    const deleteItem = vi.fn().mockResolvedValue(undefined);
    const feature = createFeatureSelection('jobs', () => ids, { deleteItem });
    feature.selectAll();

    await feature.headerProps.onDeleteSelection?.();

    expect(deleteItem.mock.calls.map(([id]) => id as string)).toEqual(ids);
    expect(feature.selectedCount).toBe(0);
  });

  it('still clears the selection when one delete rejects', async () => {
    const deleteItem = vi
      .fn()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error('boom'))
      .mockResolvedValueOnce(undefined);
    const feature = createFeatureSelection('jobs', () => ids, { deleteItem });
    feature.selectAll();

    await feature.headerProps.onDeleteSelection?.();

    expect(deleteItem).toHaveBeenCalledTimes(3);
    expect(feature.selectedCount).toBe(0);
  });
});
