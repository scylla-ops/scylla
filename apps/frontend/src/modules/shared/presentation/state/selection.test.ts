// @vitest-environment node
import { describe, it, expect, beforeEach } from 'vitest';
import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { createSelection } from './selection.svelte.ts';

/**
 * No DOM: `createSelection` is a view over a store, and `toRune` falls through
 * to a plain read outside a reactive context. The rules it pins are the same
 * six `use-selection.test.ts` holds the React one to.
 */
beforeEach(() => selectionStore.setState({ selectedIds: {} }));

describe('createSelection', () => {
  it('starts with no selection for a fresh key', () => {
    expect(createSelection('jobs').selectedIds).toEqual([]);
  });

  it('adds an id on select()', () => {
    const selection = createSelection('jobs');

    selection.select('a');

    expect(selection.selectedIds).toEqual(['a']);
  });

  it('toggles an already-selected id off', () => {
    const selection = createSelection('jobs');

    selection.select('a');
    selection.select('a');

    expect(selection.selectedIds).toEqual([]);
  });

  it('replaces the selection wholesale on selectAll()', () => {
    const selection = createSelection('jobs');

    selection.select('a');
    selection.selectAll(['b', 'c']);

    expect(selection.selectedIds).toEqual(['b', 'c']);
  });

  it('empties it on clearSelection()', () => {
    const selection = createSelection('jobs');

    selection.selectAll(['b', 'c']);
    selection.clearSelection();

    expect(selection.selectedIds).toEqual([]);
  });

  it('keeps two keys independent', () => {
    const jobs = createSelection('jobs');
    const users = createSelection('users');

    jobs.select('a');

    expect(jobs.selectedIds).toEqual(['a']);
    expect(users.selectedIds).toEqual([]);
  });
});
