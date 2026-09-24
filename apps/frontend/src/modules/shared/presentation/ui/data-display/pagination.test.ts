// @vitest-environment node
import { describe, it, expect } from 'vitest';
import { generatePageNumbers } from './pagination.ts';

/**
 * The windowing rules, without a DOM. `Pagination.svelte` only has to put these
 * on screen; which numbers they are is decided here.
 */
describe('generatePageNumbers', () => {
  it('lists every page while there are seven or fewer', () => {
    expect(generatePageNumbers(3, 7)).toEqual([1, 2, 3, 4, 5, 6, 7]);
  });

  it('keeps the first four and the last page near the start', () => {
    expect(generatePageNumbers(2, 20)).toEqual([1, 2, 3, 4, 'ellipsis', 20]);
  });

  it('keeps the first page and the last four near the end', () => {
    expect(generatePageNumbers(19, 20)).toEqual([1, 'ellipsis', 17, 18, 19, 20]);
  });

  it('slides a window around the current page in the middle, with a gap on both sides', () => {
    expect(generatePageNumbers(10, 20)).toEqual([1, 'ellipsis', 9, 10, 11, 'ellipsis', 20]);
  });

  it('handles a single page', () => {
    expect(generatePageNumbers(1, 1)).toEqual([1]);
  });
});
